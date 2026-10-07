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

            BridgeCommand::ProjectStop {
                params: _,
                responder,
            } => {
                *shutdown_requested = true;
                let _ = responder.send(Ok(ProjectStopResult { stopped: true }));
            }
        }
    }
}

/// Renders the world into an offscreen target and produces a PNG screenshot (base64 or file).
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
