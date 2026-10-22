//! Integration test covering the WebSocket JSON-RPC / MCP Bridge server.
//!
//! Verifies:
//! 1. Connection with invalid token is rejected.
//! 2. Ping command returns pong.
//! 3. Creating an entity via scene.create_entity.
//! 4. Reading the tree via scene.get_tree reflects the newly created entity.
//! 5. Capturing a viewport screenshot via game.screenshot (wgpu fallback adapter or graceful skip).
//! 6. Stopping the engine cleanly via project.stop.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use forge_assets::TextureStore;
use forge_bridge::{create_bridge_channel, BridgeServer};
use forge_core::{GameLoop, World};
use forge_render::{OffscreenTarget, RenderContext, RenderError, RenderOptions, Renderer2D};
use forge_runtime::bridge_ops::process_bridge_commands;
use forge_runtime::render_scene::{render_world_scene, DEFAULT_CLEAR_COLOR};

#[tokio::test]
async fn test_bridge_full_lifecycle_and_security() {
    let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();

    // 1. Initialize GPU context with fallback adapter
    let render_options = RenderOptions {
        force_fallback_adapter: true,
        backends: wgpu::Backends::all(),
    };

    let gpu_context = match RenderContext::new(render_options) {
        Ok(ctx) => Some(Arc::new(ctx)),
        Err(RenderError::NoAdapterFound) => {
            eprintln!("SKIPPING GPU screenshot portion: No compatible wgpu adapter found.");
            None
        }
        Err(e) => {
            eprintln!("SKIPPING GPU screenshot portion: Failed to initialize GPU adapter: {e}");
            None
        }
    };

    // 2. Set up channel and start bridge server
    let (cmd_tx, mut cmd_rx) = create_bridge_channel();
    let auth_token = "secret-token-1234567890abcdef".to_string();
    let server = BridgeServer::start_background(Some(auth_token.clone()), cmd_tx)
        .expect("Failed to start bridge server");

    let server_port = server.port;
    let valid_token = server.token.clone();

    // 3. Start headless runtime simulation loop in a background thread
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let gpu_context_clone = gpu_context.clone();

    let runtime_thread = std::thread::spawn(move || {
        let mut world = World::new();
        let mut game_loop = GameLoop::new();

        let (mut renderer, mut texture_store, offscreen) = if let Some(ref ctx) = gpu_context_clone
        {
            let r = Renderer2D::new(ctx.clone());
            let mut ts = TextureStore::new(ctx).expect("TextureStore init failed");
            ts.set_base_dir(std::path::Path::new("."));
            let ot =
                OffscreenTarget::new(&ctx.device, 64, 64).expect("OffscreenTarget init failed");
            (Some(r), Some(ts), Some(ot))
        } else {
            (None, None, None)
        };

        let frame_delta = Duration::from_secs_f64(1.0 / 60.0);
        let mut shutdown_requested = false;

        while running_clone.load(Ordering::Relaxed) && !shutdown_requested {
            game_loop.step(&mut world, frame_delta);

            if let (Some(ref mut r), Some(ref mut ts), Some(ref ctx)) =
                (&mut renderer, &mut texture_store, &gpu_context_clone)
            {
                process_bridge_commands(
                    &mut cmd_rx,
                    &mut world,
                    r,
                    ts,
                    ctx,
                    64,
                    64,
                    &mut shutdown_requested,
                );

                if let Some(ref target) = offscreen {
                    let _ = render_world_scene(
                        &world,
                        r,
                        ts,
                        ctx,
                        &target.view,
                        target.format,
                        64.0,
                        64.0,
                        Some(DEFAULT_CLEAR_COLOR),
                    );
                }
            } else {
                // If no GPU adapter, drain commands using a dummy context if possible or wait
                // We provide fallback dummy handling if no GPU
                while let Ok(cmd) = cmd_rx.try_recv() {
                    match cmd {
                        forge_bridge::BridgeCommand::EnginePing { params, responder } => {
                            let _ = responder.send(Ok(forge_protocol::EnginePingResult {
                                status: "pong".to_string(),
                                timestamp: params.timestamp,
                            }));
                        }
                        forge_bridge::BridgeCommand::SceneGetTree { params, responder } => {
                            let res = forge_core::scene_ops::scene_get_tree(&world, &params)
                                .map_err(|e| e.to_string());
                            let _ = responder.send(res);
                        }
                        forge_bridge::BridgeCommand::SceneCreateEntity { params, responder } => {
                            let res =
                                forge_core::scene_ops::scene_create_entity(&mut world, &params)
                                    .map_err(|e| e.to_string());
                            let _ = responder.send(res);
                        }
                        forge_bridge::BridgeCommand::SceneSetComponent { params, responder } => {
                            let res =
                                forge_core::scene_ops::scene_set_component(&mut world, &params)
                                    .map_err(|e| e.to_string());
                            let _ = responder.send(res);
                        }
                        forge_bridge::BridgeCommand::SceneDeleteEntity { params, responder } => {
                            let res =
                                forge_core::scene_ops::scene_delete_entity(&mut world, &params)
                                    .map_err(|e| e.to_string());
                            let _ = responder.send(res);
                        }
                        forge_bridge::BridgeCommand::GameScreenshot {
                            params: _,
                            responder,
                        } => {
                            let _ = responder
                                .send(Err("No GPU adapter available for screenshot".to_string()));
                        }
                        forge_bridge::BridgeCommand::GameSimulateInput { params, responder } => {
                            let res = forge_runtime::bridge_ops::apply_simulated_input(
                                &mut world, &params,
                            );
                            let _ = responder.send(Ok(res));
                        }
                        forge_bridge::BridgeCommand::ProjectStop {
                            params: _,
                            responder,
                        } => {
                            shutdown_requested = true;
                            let _ = responder
                                .send(Ok(forge_protocol::ProjectStopResult { stopped: true }));
                        }
                        forge_bridge::BridgeCommand::AssetList { responder, .. } => {
                            let _ = responder
                                .send(Err("Assets not initialized in dummy fallback".to_string()));
                        }
                        forge_bridge::BridgeCommand::AssetGetInfo { responder, .. } => {
                            let _ = responder
                                .send(Err("Assets not initialized in dummy fallback".to_string()));
                        }
                        forge_bridge::BridgeCommand::AssetImport { responder, .. } => {
                            let _ = responder
                                .send(Err("Assets not initialized in dummy fallback".to_string()));
                        }
                        forge_bridge::BridgeCommand::PhysicsRaycast { params, responder } => {
                            let res = forge_runtime::bridge_ops::handle_physics_raycast(&world, &params);
                            let _ = responder.send(res);
                        }
                    }
                }
            }

            if shutdown_requested {
                break;
            }

            std::thread::sleep(Duration::from_millis(5));
        }
    });

    // 4. Test Security: Verify connection with wrong token is rejected
    let invalid_ws_url = format!("ws://127.0.0.1:{}/?token=wrong_auth_token", server_port);
    let invalid_conn = connect_async(&invalid_ws_url).await;
    assert!(
        invalid_conn.is_err(),
        "Expected bridge connection to be rejected when using invalid token"
    );

    // 5. Connect client with valid token
    let valid_ws_url = format!("ws://127.0.0.1:{}/?token={}", server_port, valid_token);
    let (ws_stream, _) = connect_async(&valid_ws_url)
        .await
        .expect("Valid token client connection failed");
    let (mut write, mut read) = ws_stream.split();

    // 6. Test ping -> pong
    let ping_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "engine.ping",
        "params": {}
    });
    write
        .send(Message::Text(ping_req.to_string().into()))
        .await
        .unwrap();

    let reply = read.next().await.unwrap().unwrap();
    if let Message::Text(text) = reply {
        let val: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(val["id"], 1);
        assert_eq!(val["result"]["status"], "pong");
    } else {
        panic!("Expected text frame reply for ping");
    }

    // 7. Test scene.create_entity
    let create_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "scene.create_entity",
        "params": {
            "name": "IntegrationTestEntity"
        }
    });
    write
        .send(Message::Text(create_req.to_string().into()))
        .await
        .unwrap();

    let reply = read.next().await.unwrap().unwrap();
    let created_id = if let Message::Text(text) = reply {
        let val: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(val["id"], 2);
        let id = val["result"]["entity_id"]
            .as_str()
            .expect("entity_id string")
            .to_string();
        assert!(!id.is_empty());
        id
    } else {
        panic!("Expected text frame reply for scene.create_entity");
    };

    // 8. Test scene.get_tree
    let tree_req = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "scene.get_tree",
        "params": {}
    });
    write
        .send(Message::Text(tree_req.to_string().into()))
        .await
        .unwrap();

    let reply = read.next().await.unwrap().unwrap();
    if let Message::Text(text) = reply {
        let val: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(val["id"], 3);
        let roots = val["result"]["root_entities"]
            .as_array()
            .expect("root_entities array");
        let found = roots
            .iter()
            .any(|e| e["id"] == created_id && e["name"] == "IntegrationTestEntity");
        assert!(found, "Created entity not found in scene tree");
    } else {
        panic!("Expected text frame reply for scene.get_tree");
    }

    // 9. Test game.screenshot (if GPU adapter available)
    if gpu_context.is_some() {
        let shot_req = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "game.screenshot",
            "params": {
                "format": "png",
                "width": 64,
                "height": 64
            }
        });
        write
            .send(Message::Text(shot_req.to_string().into()))
            .await
            .unwrap();

        let reply = read.next().await.unwrap().unwrap();
        if let Message::Text(text) = reply {
            let val: Value = serde_json::from_str(&text).unwrap();
            assert_eq!(val["id"], 4);
            let b64 = val["result"]["data_base64"]
                .as_str()
                .expect("data_base64 string");
            assert!(!b64.is_empty());
            assert_eq!(val["result"]["width"], 64);
            assert_eq!(val["result"]["height"], 64);
            assert_eq!(val["result"]["format"], "png");
        } else {
            panic!("Expected text frame reply for game.screenshot");
        }
    }

    // 10. Test MCP compatibility: tools/list and tools/call
    let mcp_list_req = json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/list",
        "params": {}
    });
    write
        .send(Message::Text(mcp_list_req.to_string().into()))
        .await
        .unwrap();

    let reply = read.next().await.unwrap().unwrap();
    if let Message::Text(text) = reply {
        let val: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(val["id"], 5);
        let tools = val["result"]["tools"].as_array().expect("tools array");
        assert!(tools.iter().any(|t| t["name"] == "scene.get_tree"));
    } else {
        panic!("Expected text frame reply for tools/list");
    }

    // 11. Test project.stop
    let stop_req = json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "project.stop",
        "params": {}
    });
    write
        .send(Message::Text(stop_req.to_string().into()))
        .await
        .unwrap();

    let reply = read.next().await.unwrap().unwrap();
    if let Message::Text(text) = reply {
        let val: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(val["id"], 6);
        assert_eq!(val["result"]["stopped"], true);
    } else {
        panic!("Expected text frame reply for project.stop");
    }

    running.store(false, Ordering::Relaxed);
    let _ = runtime_thread.join();
}
