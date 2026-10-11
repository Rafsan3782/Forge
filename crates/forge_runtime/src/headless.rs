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

    let scene_path = Path::new(&args.scene);
    let scene_dir = scene_path.parent().unwrap_or(Path::new("."));
    let assets_dir = scene_dir.join("assets");

    // Initialize and scan AssetDatabase
    let mut asset_db = forge_assets::AssetDatabase::new(&assets_dir);
    if let Err(e) = asset_db.scan_and_index() {
        tracing::warn!("Failed to scan asset database at {:?}: {}", assets_dir, e);
    }
    texture_store.set_database(asset_db.clone());

    let mut animation_store = forge_assets::AnimationStore::new();
    animation_store.set_base_dir(&assets_dir);
    let mut sheet_store = forge_assets::SpriteSheetStore::new();
    sheet_store.set_base_dir(&assets_dir);

    let scene = Scene::load_from_file(&args.scene)?;
    let mut world = World::new();
    scene.spawn_into_world(&mut world)?;

    // Insert asset resources
    world.insert_resource(asset_db);
    world.insert_resource(animation_store);
    world.insert_resource(sheet_store);
    world.insert_resource(forge_core::EventBus::new());

    // Initialize ActionMap
    let action_map = if let Some(ref map_path) = args.input_map {
        forge_input::ActionMap::load_from_file(map_path)?
    } else {
        let candidate = scene_dir.join("input_map.json");
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
    game_loop.add_update_system(forge_assets::animator_update_system);

    if args.demo_input {
        game_loop.add_update_system(crate::demo_input::demo_hero_movement_system);
    }

    // Start hot reload watcher if directory exists
    let mut watcher = if assets_dir.exists() {
        forge_assets::HotReloadWatcher::new(&assets_dir).ok()
    } else {
        None
    };

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

            // Poll hot reload events
            if let Some(ref mut w) = watcher {
                for event in w.poll_events() {
                    match event {
                        forge_assets::HotReloadEvent::Texture(p) => {
                            let path_str = p.to_string_lossy();
                            let asset_id = if let Some(db) =
                                world.get_resource::<forge_assets::AssetDatabase>()
                            {
                                db.get_id_by_path(&p).unwrap_or(&path_str).to_string()
                            } else {
                                path_str.to_string()
                            };
                            let _ = texture_store.reload(&asset_id, &context);
                            tracing::info!("Hot reloaded texture: {:?}", p);
                        }
                        forge_assets::HotReloadEvent::SpriteSheet(p) => {
                            if let Some(mut sheets) =
                                world.get_resource_mut::<forge_assets::SpriteSheetStore>()
                            {
                                let path_str = p.to_string_lossy();
                                let _ = sheets.reload(&path_str, &p);
                                tracing::info!("Hot reloaded spritesheet: {:?}", p);
                            }
                        }
                        forge_assets::HotReloadEvent::Animation(p) => {
                            if let Some(mut anims) =
                                world.get_resource_mut::<forge_assets::AnimationStore>()
                            {
                                let path_str = p.to_string_lossy();
                                let _ = anims.reload(&path_str, &p);
                                tracing::info!("Hot reloaded animation: {:?}", p);
                            }
                        }
                        forge_assets::HotReloadEvent::Scene(p) => {
                            if let Ok(new_scene) = Scene::load_from_file(&p) {
                                // Clear world and re-spawn
                                world.clear_entities();
                                let _ = new_scene.spawn_into_world(&mut world);
                                tracing::info!("Hot reloaded scene from {:?}", p);
                            }
                        }
                        forge_assets::HotReloadEvent::Meta(p) => {
                            if let Some(mut db) =
                                world.get_resource_mut::<forge_assets::AssetDatabase>()
                            {
                                let _ = db.scan_and_index();
                                texture_store.set_database(db.clone());
                                tracing::info!("Hot reloaded asset database metadata: {:?}", p);
                            }
                        }
                        _ => {}
                    }
                }
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
