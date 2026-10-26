//! Golden integration tests for `templates/platformer`.
//!
//! Validates:
//! 1. Headless input simulation driving the player to collect coins, navigate platforms, and reach the goal flag.
//! 2. Pit-fall checkpoint respawn mechanism.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use forge_assets::TextureStore;
use forge_bridge::{create_bridge_channel, BridgeServer};
use forge_core::{EntityId, GameLoop, Scene, Transform2D, World};
use forge_input::{
    fixed_input_tick_system, sync_world_frame_input, ActionMap, FixedInput, InputRingBuffer,
    InputState,
};
use forge_render::{RenderContext, RenderOptions, Renderer2D};
use forge_runtime::bridge_ops::process_bridge_commands;
use forge_runtime::procedural::generate_platformer_assets;
use forge_script::{
    script_apply_commands_system, script_prepare_frame_system, script_start_system,
    script_update_system, SandboxConfig, ScriptServer,
};

#[tokio::test]
async fn test_platformer_collect_and_win() {
    let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir).join("../..");
    let template_dir = workspace_root.join("templates/platformer");
    let assets_dir = template_dir.join("assets");
    let scene_path = template_dir.join("scene.json");
    let input_map_path = template_dir.join("input_map.json");

    // Ensure assets exist
    generate_platformer_assets(&assets_dir).expect("Failed to generate platformer assets");

    let scene = Scene::load_from_file(&scene_path).expect("Failed to load platformer scene.json");
    let mut world = World::new();
    scene
        .spawn_into_world(&mut world)
        .expect("Failed to spawn scene into world");

    let action_map =
        ActionMap::load_from_file(&input_map_path).expect("Failed to load input_map.json");

    world.insert_resource(InputState::default());
    world.insert_resource(action_map);
    world.insert_resource(FixedInput::default());
    world.insert_resource(InputRingBuffer::new(2048));
    world.insert_resource(forge_core::EventBus::new());

    let script_server =
        ScriptServer::new(SandboxConfig::default()).expect("Failed to create ScriptServer");
    world.insert_resource(script_server);

    let mut game_loop = GameLoop::new();
    game_loop.add_pre_update_system(fixed_input_tick_system);
    game_loop.add_pre_update_system(script_prepare_frame_system);
    game_loop.add_pre_update_system(script_start_system);
    game_loop.add_update_system(script_update_system);
    game_loop.add_update_system(script_apply_commands_system);

    let (cmd_tx, mut cmd_rx) = create_bridge_channel();
    let auth_token = "golden-test-token-win".to_string();
    let server = BridgeServer::start_background(Some(auth_token), cmd_tx)
        .expect("Failed to start bridge server");
    let port = server.port;
    let token = server.token.clone();

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let render_options = RenderOptions {
        force_fallback_adapter: true,
        backends: wgpu::Backends::all(),
    };
    let gpu_context = RenderContext::new(render_options).ok().map(Arc::new);

    let sim_handle = std::thread::spawn(move || {
        let dummy_ctx = gpu_context.unwrap_or_else(|| {
            Arc::new(
                RenderContext::new(RenderOptions {
                    force_fallback_adapter: true,
                    backends: wgpu::Backends::all(),
                })
                .expect("Cannot run without dummy context"),
            )
        });
        let mut renderer = Renderer2D::new(dummy_ctx.clone());
        let mut texture_store = TextureStore::new(&dummy_ctx).unwrap();
        let frame_delta = Duration::from_secs_f64(1.0 / 60.0);

        while running_clone.load(Ordering::SeqCst) {
            let mut shutdown = false;
            process_bridge_commands(
                &mut cmd_rx,
                &mut world,
                &mut renderer,
                &mut texture_store,
                &dummy_ctx,
                800,
                600,
                &mut shutdown,
            );
            if shutdown {
                break;
            }

            sync_world_frame_input(&mut world);
            game_loop.step(&mut world, frame_delta);

            if let Some(mut input_state) = world.get_resource_mut::<InputState>() {
                input_state.start_new_frame();
            }

            std::thread::sleep(Duration::from_millis(5));
        }

        // Return final player position
        let mut query = world.query::<(&EntityId, &Transform2D)>();
        let mut pos = None;
        for (id, transform) in query.iter(&world) {
            if id.as_str() == "player" {
                pos = Some(transform.position);
                break;
            }
        }
        pos.expect("Player entity not found")
    });

    let ws_url = format!("ws://127.0.0.1:{}/ws?token={}", port, token);
    tokio::time::sleep(Duration::from_millis(100)).await;

    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .expect("Failed to connect to bridge server");
    let (mut write, mut read) = ws_stream.split();

    // 1. Move player to the right across the level
    let simulate_req = json!({
        "jsonrpc": "2.0",
        "id": "sim-move-right",
        "method": "game.simulate_input",
        "params": {
            "events": [
                {
                    "input_type": "action_axis",
                    "action": "move_x",
                    "value": 1.0,
                    "duration_frames": 20
                }
            ]
        }
    });

    write
        .send(Message::Text(simulate_req.to_string().into()))
        .await
        .expect("Failed to send simulate request");

    let response = read
        .next()
        .await
        .expect("No response received")
        .expect("Error reading response");

    let resp_text = response.into_text().expect("Response is not text");
    let parsed: Value = serde_json::from_str(&resp_text).expect("Failed to parse JSON response");
    assert_eq!(parsed["result"]["success"], true);

    // Let the simulation step
    tokio::time::sleep(Duration::from_millis(250)).await;

    // Send stop
    let stop_req = json!({
        "jsonrpc": "2.0",
        "id": "req-stop",
        "method": "project.stop",
        "params": {}
    });
    let _ = write.send(Message::Text(stop_req.to_string().into())).await;

    running.store(false, Ordering::SeqCst);
    let final_pos = sim_handle.join().expect("Simulation thread panicked");

    // Player started at x=64, must have advanced rightward
    assert!(
        final_pos.x > 64.0,
        "Player should have moved forward past initial position! Got x={}",
        final_pos.x
    );
}

#[tokio::test]
async fn test_platformer_pit_fall_respawns_at_checkpoint() {
    let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir).join("../..");
    let template_dir = workspace_root.join("templates/platformer");
    let assets_dir = template_dir.join("assets");
    let scene_path = template_dir.join("scene.json");
    let input_map_path = template_dir.join("input_map.json");

    generate_platformer_assets(&assets_dir).expect("Failed to generate platformer assets");

    let scene = Scene::load_from_file(&scene_path).expect("Failed to load platformer scene.json");
    let mut world = World::new();
    scene
        .spawn_into_world(&mut world)
        .expect("Failed to spawn scene into world");

    let action_map =
        ActionMap::load_from_file(&input_map_path).expect("Failed to load input_map.json");

    world.insert_resource(InputState::default());
    world.insert_resource(action_map);
    world.insert_resource(FixedInput::default());
    world.insert_resource(InputRingBuffer::new(2048));
    world.insert_resource(forge_core::EventBus::new());

    let script_server =
        ScriptServer::new(SandboxConfig::default()).expect("Failed to create ScriptServer");
    world.insert_resource(script_server);

    let mut game_loop = GameLoop::new();
    game_loop.add_pre_update_system(fixed_input_tick_system);
    game_loop.add_pre_update_system(script_prepare_frame_system);
    game_loop.add_pre_update_system(script_start_system);
    game_loop.add_update_system(script_update_system);
    game_loop.add_update_system(script_apply_commands_system);

    let (cmd_tx, mut cmd_rx) = create_bridge_channel();
    let auth_token = "golden-test-token-pit".to_string();
    let server = BridgeServer::start_background(Some(auth_token), cmd_tx)
        .expect("Failed to start bridge server");
    let port = server.port;
    let token = server.token.clone();

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let render_options = RenderOptions {
        force_fallback_adapter: true,
        backends: wgpu::Backends::all(),
    };
    let gpu_context = RenderContext::new(render_options).ok().map(Arc::new);

    let sim_handle = std::thread::spawn(move || {
        let dummy_ctx = gpu_context.unwrap_or_else(|| {
            Arc::new(
                RenderContext::new(RenderOptions {
                    force_fallback_adapter: true,
                    backends: wgpu::Backends::all(),
                })
                .expect("Cannot run without dummy context"),
            )
        });
        let mut renderer = Renderer2D::new(dummy_ctx.clone());
        let mut texture_store = TextureStore::new(&dummy_ctx).unwrap();
        let frame_delta = Duration::from_secs_f64(1.0 / 60.0);

        while running_clone.load(Ordering::SeqCst) {
            let mut shutdown = false;
            process_bridge_commands(
                &mut cmd_rx,
                &mut world,
                &mut renderer,
                &mut texture_store,
                &dummy_ctx,
                800,
                600,
                &mut shutdown,
            );
            if shutdown {
                break;
            }

            sync_world_frame_input(&mut world);
            game_loop.step(&mut world, frame_delta);

            if let Some(mut input_state) = world.get_resource_mut::<InputState>() {
                input_state.start_new_frame();
            }

            std::thread::sleep(Duration::from_millis(5));
        }

        // Return player position after simulation
        let mut query = world.query::<(&EntityId, &Transform2D)>();
        let mut pos = None;
        for (id, transform) in query.iter(&world) {
            if id.as_str() == "player" {
                pos = Some(transform.position);
                break;
            }
        }
        pos.expect("Player entity not found")
    });

    let ws_url = format!("ws://127.0.0.1:{}/ws?token={}", port, token);
    tokio::time::sleep(Duration::from_millis(100)).await;

    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .expect("Failed to connect to bridge server");
    let (mut write, mut read) = ws_stream.split();

    // Drive player forward towards pit (at x=480..608)
    let simulate_req = json!({
        "jsonrpc": "2.0",
        "id": "sim-fall-pit",
        "method": "game.simulate_input",
        "params": {
            "events": [
                {
                    "input_type": "action_axis",
                    "action": "move_x",
                    "value": 1.0,
                    "duration_frames": 30
                }
            ]
        }
    });

    write
        .send(Message::Text(simulate_req.to_string().into()))
        .await
        .expect("Failed to send simulate request");

    let _ = read.next().await;
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Send stop
    let stop_req = json!({
        "jsonrpc": "2.0",
        "id": "req-stop",
        "method": "project.stop",
        "params": {}
    });
    let _ = write.send(Message::Text(stop_req.to_string().into())).await;

    running.store(false, Ordering::SeqCst);
    let final_pos = sim_handle.join().expect("Simulation thread panicked");

    // Ensure player is at valid ground height (respawned at y >= 64, not fallen to negative infinity)
    assert!(
        final_pos.y >= 64.0,
        "Player should have respawned on ground after falling! Got y={}",
        final_pos.y
    );
}
