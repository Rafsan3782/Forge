//! Headless offscreen renderer executing fixed simulation steps and capturing PNG screenshots.

use crate::cli::CliArgs;
use crate::render_scene::{render_world_scene, DEFAULT_CLEAR_COLOR};
use forge_assets::TextureStore;
use forge_core::{GameLoop, Scene, World};
use forge_render::{OffscreenTarget, RenderContext, RenderOptions, Renderer2D};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// Executes the headless simulation loop without opening an interactive window.
///
/// Runs N simulation frames of the fixed timestep loop from `forge_core`, renders to an offscreen
/// texture, and optionally exports a PNG screenshot.
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

    let mut game_loop = GameLoop::new();
    let frame_count = args.frames.max(1);
    let frame_delta = Duration::from_secs_f64(1.0 / 60.0);

    for frame_idx in 0..frame_count {
        game_loop.step(&mut world, frame_delta);

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

    Ok(())
}
