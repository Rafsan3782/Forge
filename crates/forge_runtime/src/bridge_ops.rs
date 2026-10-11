//! Bridge command execution between frames on the engine thread.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use forge_assets::TextureStore;
use forge_bridge::{BridgeCommand, BridgeCommandReceiver};
use forge_core::scene_ops::{
    scene_create_entity, scene_delete_entity, scene_get_tree, scene_set_component,
};
use forge_core::World;
use forge_protocol::{
    EnginePingResult, GameScreenshotParams, GameScreenshotResult, ProjectStopResult,
};
use forge_render::{OffscreenTarget, RenderContext, Renderer2D};
use image::ImageEncoder;

use crate::render_scene::{render_world_scene, DEFAULT_CLEAR_COLOR};

/// Helper to get current timestamp in milliseconds.
pub fn current_timestamp_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
}

/// Drains pending bridge commands from the queue without holding locks across frames.
#[allow(clippy::too_many_arguments)]
pub fn process_bridge_commands(
    command_rx: &mut BridgeCommandReceiver,
    world: &mut World,
    renderer: &mut Renderer2D,
    texture_store: &mut TextureStore,
    context: &RenderContext,
    default_width: u32,
    default_height: u32,
    shutdown_requested: &mut bool,
) {
    while let Ok(cmd) = command_rx.try_recv() {
        match cmd {
            BridgeCommand::EnginePing { params, responder } => {
                let timestamp = if params.timestamp != 0.0 {
                    params.timestamp
                } else {
                    current_timestamp_ms()
                };
                let _ = responder.send(Ok(EnginePingResult {
                    status: "pong".to_string(),
                    timestamp,
                }));
            }

            BridgeCommand::SceneGetTree { params, responder } => {
                let res = scene_get_tree(world, &params).map_err(|e| e.to_string());
                let _ = responder.send(res);
            }

            BridgeCommand::SceneCreateEntity { params, responder } => {
                let res = scene_create_entity(world, &params).map_err(|e| e.to_string());
                let _ = responder.send(res);
            }

            BridgeCommand::SceneSetComponent { params, responder } => {
                let res = scene_set_component(world, &params).map_err(|e| e.to_string());
                let _ = responder.send(res);
            }

            BridgeCommand::SceneDeleteEntity { params, responder } => {
                let res = scene_delete_entity(world, &params).map_err(|e| e.to_string());
                let _ = responder.send(res);
            }

            BridgeCommand::GameScreenshot { params, responder } => {
                let res = capture_screenshot(
                    world,
                    renderer,
                    texture_store,
                    context,
                    &params,
                    default_width,
                    default_height,
                );
                let _ = responder.send(res);
            }

            BridgeCommand::GameSimulateInput { params, responder } => {
                let res = apply_simulated_input(world, &params);
                let _ = responder.send(Ok(res));
            }

            BridgeCommand::ProjectStop {
                params: _,
                responder,
            } => {
                *shutdown_requested = true;
                let _ = responder.send(Ok(ProjectStopResult { stopped: true }));
            }

            BridgeCommand::AssetList { params, responder } => {
                let res = handle_asset_list(world, &params);
                let _ = responder.send(res);
            }

            BridgeCommand::AssetGetInfo { params, responder } => {
                let res = handle_asset_get_info(world, &params);
                let _ = responder.send(res);
            }

            BridgeCommand::AssetImport { params, responder } => {
                let res = handle_asset_import(world, &params);
                let _ = responder.send(res);
            }
        }
    }
}

fn handle_asset_list(
    world: &World,
    params: &forge_protocol::AssetListParams,
) -> Result<forge_protocol::AssetListResult, String> {
    let db = world
        .get_resource::<forge_assets::AssetDatabase>()
        .ok_or_else(|| "AssetDatabase resource not initialized in world".to_string())?;

    let filter_type = params.asset_type.as_deref();
    let mut assets = Vec::new();

    for entry in db.all_entries() {
        let type_str = match &entry.meta.asset_type {
            forge_assets::AssetType::Texture => "texture".to_string(),
            forge_assets::AssetType::SpriteSheet => "spritesheet".to_string(),
            forge_assets::AssetType::Animation => "animation".to_string(),
            forge_assets::AssetType::Scene => "scene".to_string(),
            forge_assets::AssetType::Audio => "audio".to_string(),
            forge_assets::AssetType::Other(s) => s.clone(),
        };

        if let Some(ft) = filter_type {
            if !ft.is_empty() && !type_str.eq_ignore_ascii_case(ft) {
                continue;
            }
        }

        assets.push(forge_protocol::AssetInfo {
            id: entry.id.clone(),
            path: entry.file_path.to_string_lossy().to_string(),
            asset_type: type_str,
            metadata: entry.meta.import_settings.clone(),
        });
    }

    Ok(forge_protocol::AssetListResult { assets })
}

fn handle_asset_get_info(
    world: &World,
    params: &forge_protocol::AssetGetInfoParams,
) -> Result<forge_protocol::AssetGetInfoResult, String> {
    let db = world
        .get_resource::<forge_assets::AssetDatabase>()
        .ok_or_else(|| "AssetDatabase resource not initialized in world".to_string())?;

    let entry = db
        .get(&params.id)
        .ok_or_else(|| format!("Asset with id '{}' not found", params.id))?;

    let type_str = match &entry.meta.asset_type {
        forge_assets::AssetType::Texture => "texture".to_string(),
        forge_assets::AssetType::SpriteSheet => "spritesheet".to_string(),
        forge_assets::AssetType::Animation => "animation".to_string(),
        forge_assets::AssetType::Scene => "scene".to_string(),
        forge_assets::AssetType::Audio => "audio".to_string(),
        forge_assets::AssetType::Other(s) => s.clone(),
    };

    Ok(forge_protocol::AssetGetInfoResult {
        id: entry.id.clone(),
        path: entry.file_path.to_string_lossy().to_string(),
        asset_type: type_str,
        metadata: entry.meta.import_settings.clone(),
    })
}

fn handle_asset_import(
    world: &mut World,
    params: &forge_protocol::AssetImportParams,
) -> Result<forge_protocol::AssetImportResult, String> {
    let mut db = world
        .get_resource_mut::<forge_assets::AssetDatabase>()
        .ok_or_else(|| "AssetDatabase resource not initialized in world".to_string())?;

    let source_path = std::path::Path::new(&params.source_path);
    let target_path = std::path::Path::new(&params.target_path);

    let explicit_type = params
        .asset_type
        .as_deref()
        .map(|t| match t.to_lowercase().as_str() {
            "texture" => forge_assets::AssetType::Texture,
            "spritesheet" => forge_assets::AssetType::SpriteSheet,
            "animation" => forge_assets::AssetType::Animation,
            "scene" => forge_assets::AssetType::Scene,
            "audio" => forge_assets::AssetType::Audio,
            other => forge_assets::AssetType::Other(other.to_string()),
        });

    let entry = db
        .import_file(
            source_path,
            target_path,
            explicit_type,
            params.metadata.clone(),
        )
        .map_err(|e| format!("Failed to import asset: {}", e))?;

    let type_str = match &entry.meta.asset_type {
        forge_assets::AssetType::Texture => "texture".to_string(),
        forge_assets::AssetType::SpriteSheet => "spritesheet".to_string(),
        forge_assets::AssetType::Animation => "animation".to_string(),
        forge_assets::AssetType::Scene => "scene".to_string(),
        forge_assets::AssetType::Audio => "audio".to_string(),
        forge_assets::AssetType::Other(s) => s.clone(),
    };

    Ok(forge_protocol::AssetImportResult {
        id: entry.id,
        path: entry.file_path.to_string_lossy().to_string(),
        asset_type: type_str,
    })
}

/// Renders the world into an offscreen target and produces a PNG screenshot (base64 or file).
#[allow(clippy::too_many_arguments)]
pub fn capture_screenshot(
    world: &World,
    renderer: &mut Renderer2D,
    texture_store: &mut TextureStore,
    context: &RenderContext,
    params: &GameScreenshotParams,
    default_width: u32,
    default_height: u32,
) -> Result<GameScreenshotResult, String> {
    let width = params.width.unwrap_or(default_width).max(1);
    let height = params.height.unwrap_or(default_height).max(1);

    let offscreen = OffscreenTarget::new(&context.device, width, height)
        .map_err(|e| format!("Failed to create offscreen render target: {}", e))?;

    render_world_scene(
        world,
        renderer,
        texture_store,
        context,
        &offscreen.view,
        offscreen.format,
        width as f32,
        height as f32,
        Some(DEFAULT_CLEAR_COLOR),
    )
    .map_err(|e| format!("Failed to render scene for screenshot: {}", e))?;

    let rgba_bytes = offscreen
        .read_rgba(&context.device, &context.queue)
        .map_err(|e| format!("Failed to readback screenshot buffer: {}", e))?;

    let mut png_bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    encoder
        .write_image(&rgba_bytes, width, height, image::ExtendedColorType::Rgba8)
        .map_err(|e| format!("Failed to encode PNG screenshot: {}", e))?;

    let file_path = if let Some(ref path_str) = params.file_path {
        let path = Path::new(path_str);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory for screenshot: {}", e))?;
            }
        }
        std::fs::write(path, &png_bytes)
            .map_err(|e| format!("Failed to write screenshot to '{}': {}", path_str, e))?;
        Some(path_str.clone())
    } else {
        None
    };

    let data_base64 = if file_path.is_none() {
        Some(base64::engine::general_purpose::STANDARD.encode(&png_bytes))
    } else {
        None
    };

    Ok(GameScreenshotResult {
        data_base64,
        file_path,
        width,
        height,
        format: "png".to_string(),
    })
}

/// Applies simulated input events to the World's InputState and records into InputRingBuffer.
pub fn apply_simulated_input(
    world: &mut World,
    params: &forge_protocol::GameSimulateInputParams,
) -> forge_protocol::GameSimulateInputResult {
    use forge_input::{InputRingBuffer, InputState, KeyCode, MouseButton};

    let mut applied_count = 0;

    if let Some(mut state) = world.get_resource_mut::<InputState>() {
        let frame = state.current_frame();
        for ev in &params.events {
            applied_count += 1;
            match ev.input_type.as_str() {
                "key_down" => {
                    if let Some(ref k_str) = ev.key {
                        if let Some(k) = KeyCode::from_name(k_str) {
                            state.simulate_key(k, true, ev.duration_frames);
                        }
                    }
                }
                "key_up" => {
                    if let Some(ref k_str) = ev.key {
                        if let Some(k) = KeyCode::from_name(k_str) {
                            state.simulate_key(k, false, 1);
                        }
                    }
                }
                "action_down" => {
                    if let Some(ref act) = ev.action {
                        state.simulate_action(act, true, ev.duration_frames);
                    }
                }
                "action_up" => {
                    if let Some(ref act) = ev.action {
                        state.simulate_action(act, false, 1);
                    }
                }
                "action_axis" => {
                    if let Some(ref act) = ev.action {
                        let val = ev.value.unwrap_or(0.0) as f32;
                        state.simulate_axis(act, val, ev.duration_frames);
                    }
                }
                "mouse_move" => {
                    let val = ev.value.unwrap_or(0.0) as f32;
                    state.feed_mouse_move(forge_core::Vec2::new(val, val), None);
                }
                "mouse_down" => {
                    let btn = ev
                        .key
                        .as_deref()
                        .and_then(MouseButton::from_name)
                        .unwrap_or(MouseButton::Left);
                    state.feed_mouse_button_press(btn, None);
                }
                "mouse_up" => {
                    let btn = ev
                        .key
                        .as_deref()
                        .and_then(MouseButton::from_name)
                        .unwrap_or(MouseButton::Left);
                    state.feed_mouse_button_release(btn, None);
                }
                _ => {}
            }
        }

        // Also record to ring buffer if present
        if let Some(mut buf) = world.get_resource_mut::<InputRingBuffer>() {
            for ev in &params.events {
                buf.record(
                    frame,
                    ev.input_type.clone(),
                    ev.action
                        .clone()
                        .or_else(|| ev.key.clone())
                        .unwrap_or_default(),
                    ev.value.unwrap_or(1.0) as f32,
                );
            }
        }
    }

    forge_protocol::GameSimulateInputResult {
        success: true,
        applied_count,
    }
}
