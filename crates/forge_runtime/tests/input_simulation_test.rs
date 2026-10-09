//! Integration test verifying `game.simulate_input` moves an entity in headless simulation.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use forge_assets::TextureStore;
use forge_bridge::{create_bridge_channel, BridgeServer};
use forge_core::{GameLoop, Name, Scene, Transform2D, World};
use forge_input::{
    fixed_input_tick_system, sync_world_frame_input, ActionMap, FixedInput, InputRingBuffer,
    InputState,
};
use forge_render::{RenderContext, RenderOptions, Renderer2D};
use forge_runtime::bridge_ops::process_bridge_commands;
use forge_runtime::demo_hero_movement_system;

#[tokio::test]
async fn test_simulate_input_moves_hero() {
    let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();

    // 1. Set up headless world with Hello scene
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir).join("../..");
    let scene_path = workspace_root.join("templates/hello/scene.json");
    let input_map_path = workspace_root.join("templates/hello/input_map.json");

    let scene =
        Scene::load_from_file(&scene_path).expect("Failed to load templates/hello/scene.json");
    let mut world = World::new();
    scene
        .spawn_into_world(&mut world)
        .expect("Failed to spawn scene into world");

    // Load default action map
    let action_map =
        ActionMap::load_from_file(&input_map_path).expect("Failed to load input_map.json");

    world.insert_resource(InputState::default());
    world.insert_resource(action_map);
    world.insert_resource(FixedInput::default());
    world.insert_resource(InputRingBuffer::new(2048));

    // Verify initial hero position
    let initial_pos = {
        let mut query = world.query::<(&Name, &Transform2D)>();
        let mut pos = None;
        for (name, transform) in query.iter(&world) {
            if name.as_str() == "Hero" {
                pos = Some(transform.position);
                break;
            }
        }
        pos.expect("Hero entity not found in scene")
    };
    assert_eq!(initial_pos.x, 0.0);
    assert_eq!(initial_pos.y, 0.0);

    // 2. Set up game loop with demo hero movement system
    let mut game_loop = GameLoop::new();
    game_loop.add_pre_update_system(fixed_input_tick_system);
    game_loop.add_update_system(demo_hero_movement_system);

    // 3. Set up bridge server and channel
    let (cmd_tx, mut cmd_rx) = create_bridge_channel();
    let auth_token = "input-test-token-123".to_string();
    let server = BridgeServer::start_background(Some(auth_token), cmd_tx)
        .expect("Failed to start bridge server");
    let port = server.port;
    let token = server.token.clone();

    // 4. Run headless simulation loop in background thread
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let render_options = RenderOptions {
        force_fallback_adapter: true,
        backends: wgpu::Backends::all(),
    };
    let gpu_context = RenderContext::new(render_options).ok().map(Arc::new);

    let sim_handle = std::thread::spawn(move || {
        let dummy_ctx = gpu_context.unwrap_or_else(|| {
            // If no adapter, create minimal context or use headless dummy
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

            std::thread::sleep(Duration::from_millis(16));
        }

        // Return final hero position
        let mut query = world.query::<(&Name, &Transform2D)>();
        let mut final_pos = None;
        for (name, transform) in query.iter(&world) {
            if name.as_str() == "Hero" {
                final_pos = Some(transform.position);
                break;
            }
        }
        final_pos.unwrap()
    });

    // 5. Connect via WebSocket client and send game.simulate_input
    let ws_url = format!("ws://127.0.0.1:{}/ws?token={}", port, token);
    tokio::time::sleep(Duration::from_millis(100)).await;

    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .expect("Failed to connect to bridge server");
    let (mut write, mut read) = ws_stream.split();

    // Send game.simulate_input: move right (action: move_x with value 1.0 for 10 frames)
    let simulate_req = json!({
        "jsonrpc": "2.0",
        "id": "req-sim-1",
        "method": "game.simulate_input",
        "params": {
            "events": [
                {
                    "input_type": "action_axis",
                    "action": "move_x",
                    "value": 1.0,
                    "duration_frames": 10
                }
            ]
        }
    });

    write
        .send(Message::Text(simulate_req.to_string().into()))
        .await
        .expect("Failed to send game.simulate_input request");

    let response = read
        .next()
        .await
        .expect("No response received")
        .expect("Error reading response");

    let resp_text = response.into_text().expect("Response is not text");
    let parsed: Value = serde_json::from_str(&resp_text).expect("Failed to parse JSON response");
    if let Some(err) = parsed.get("error") {
        panic!("RPC returned error: {:?}", err);
    }
    assert_eq!(parsed["id"], "req-sim-1");
    assert_eq!(parsed["result"]["success"], true);
    assert_eq!(parsed["result"]["applied_count"], 1);

    // Let the simulation step for ~15 frames (250ms)
    tokio::time::sleep(Duration::from_millis(250)).await;

    // Send project.stop
    let stop_req = json!({
        "jsonrpc": "2.0",
        "id": "req-stop",
        "method": "project.stop",
        "params": {}
    });
    let _ = write.send(Message::Text(stop_req.to_string().into())).await;

    running.store(false, Ordering::SeqCst);
    let final_pos = sim_handle.join().expect("Simulation thread panicked");

    // Hero position X must have moved to the right (x > 0.0)
    assert!(
        final_pos.x > 0.0,
        "Hero position X should have moved to the right! Got: {}",
        final_pos.x
    );
    assert_eq!(
        final_pos.y, 0.0,
        "Hero position Y should not have changed. Got: {}",
        final_pos.y
    );
}
