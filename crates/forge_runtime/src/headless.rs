//! Headless offscreen renderer executing fixed simulation steps and capturing PNG screenshots.

use crate::bridge_ops::process_bridge_commands;
use crate::cli::CliArgs;
use crate::render_scene::{render_world_scene, DEFAULT_CLEAR_COLOR};
use forge_assets::TextureStore;
use forge_bridge::{create_bridge_channel, BridgeServer};
use forge_core::{GameLoop, Scene, World};
use forge_render::{OffscreenTarget, RenderContext, RenderOptions, Renderer2D};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// Executes the headless simulation loop without opening an interactive window.
///
/// Runs N simulation frames of the fixed timestep loop from `forge_core`, renders to an offscreen
/// texture, and optionally exports a PNG screenshot. If bridge is active and no fixed screenshot
/// export was configured, runs continuously serving RPC requests until `project.stop`.
pub fn run_headless(args: &CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(
        "Running headless mode: scene='{}', frames={}, resolution={}x{}",
        args.scene,
        args.frames,
        args.width,
        args.height
    );

    let render_options = RenderOptions {
        force_fallback_adapter: args.force_fallback,
        backends: wgpu::Backends::all(),
    };

    let context = Arc::new(RenderContext::new(render_options)?);
    let mut texture_store = TextureStore::new(&context)?;
    texture_store.set_base_dir(Path::new("."));

    let mut renderer = Renderer2D::new(context.clone());
    let offscreen = OffscreenTarget::new(&context.device, args.width, args.height)?;

    let scene = Scene::load_from_file(&args.scene)?;
    let mut world = World::new();
    scene.spawn_into_world(&mut world)?;

    // Initialize ActionMap
    let action_map = if let Some(ref map_path) = args.input_map {
        forge_input::ActionMap::load_from_file(map_path)?
    } else {
        let scene_path = Path::new(&args.scene);
        let candidate = scene_path
            .parent()
            .unwrap_or(Path::new(""))
            .join("input_map.json");
        if candidate.exists() {
            forge_input::ActionMap::load_from_file(&candidate)?
        } else {
            forge_input::ActionMap::default()
        }
    };

    // Insert input resources
    world.insert_resource(forge_input::InputState::default());
    world.insert_resource(action_map);
    world.insert_resource(forge_input::FixedInput::default());
    world.insert_resource(forge_input::InputRingBuffer::new(2048));

    let mut game_loop = GameLoop::new();
    game_loop.add_pre_update_system(forge_input::fixed_input_tick_system);

    if args.demo_input {
        game_loop.add_update_system(crate::demo_input::demo_hero_movement_system);
    }

    let frame_delta = Duration::from_secs_f64(1.0 / 60.0);

    // Initialize the WebSocket bridge server if enabled
    let (mut command_rx, _bridge_server) = if !args.no_bridge {
        let (cmd_tx, cmd_rx) = create_bridge_channel();
        let server = BridgeServer::start_background(args.bridge_token.clone(), cmd_tx)?;
        (Some(cmd_rx), Some(server))
    } else {
        (None, None)
    };

    let continuous_mode = args.stay_open || (args.screenshot.is_none() && !args.no_bridge);

    if continuous_mode {
        tracing::info!("Headless mode running continuously in RPC server mode");
        let mut shutdown_requested = false;

        while !shutdown_requested {
            // Sync InputState + ActionMap -> FixedInput
            forge_input::sync_world_frame_input(&mut world);

            game_loop.step(&mut world, frame_delta);

            if let Some(mut input_state) = world.get_resource_mut::<forge_input::InputState>() {
                input_state.start_new_frame();
            }

            if let Some(ref mut rx) = command_rx {
                process_bridge_commands(
                    rx,
                    &mut world,
                    &mut renderer,
                    &mut texture_store,
                    &context,
                    args.width,
                    args.height,
                    &mut shutdown_requested,
                );
            }

            if shutdown_requested {
                tracing::info!("Clean shutdown requested via bridge project.stop");
                break;
            }

            let _ = render_world_scene(
                &world,
                &mut renderer,
                &mut texture_store,
                &context,
                &offscreen.view,
                offscreen.format,
                args.width as f32,
                args.height as f32,
                Some(DEFAULT_CLEAR_COLOR),
            );

            std::thread::sleep(Duration::from_millis(16));
        }
    } else {
        let frame_count = args.frames.max(1);
        let mut shutdown_requested = false;

        for frame_idx in 0..frame_count {
            // Sync InputState + ActionMap -> FixedInput
            forge_input::sync_world_frame_input(&mut world);

            game_loop.step(&mut world, frame_delta);

            if let Some(mut input_state) = world.get_resource_mut::<forge_input::InputState>() {
                input_state.start_new_frame();
            }

            if let Some(ref mut rx) = command_rx {
                process_bridge_commands(
                    rx,
                    &mut world,
                    &mut renderer,
                    &mut texture_store,
                    &context,
                    args.width,
                    args.height,
                    &mut shutdown_requested,
                );
            }

            render_world_scene(
                &world,
                &mut renderer,
                &mut texture_store,
                &context,
                &offscreen.view,
                offscreen.format,
                args.width as f32,
                args.height as f32,
                Some(DEFAULT_CLEAR_COLOR),
            )?;

            tracing::debug!("Rendered headless frame {}/{}", frame_idx + 1, frame_count);

            if shutdown_requested {
                break;
            }
        }

        if let Some(ref screenshot_path) = args.screenshot {
            let rgba_bytes = offscreen.read_rgba(&context.device, &context.queue)?;
            let path = Path::new(screenshot_path);
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }

            image::save_buffer(
                path,
                &rgba_bytes,
                args.width,
                args.height,
                image::ExtendedColorType::Rgba8,
            )?;

            tracing::info!("Saved headless screenshot to '{}'", screenshot_path);
        }
    }

    Ok(())
}
