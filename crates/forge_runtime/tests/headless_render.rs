//! Headless offscreen render integration test comparing against reference image.

use forge_assets::TextureStore;
use forge_core::{GameLoop, Scene, World};
use forge_render::{OffscreenTarget, RenderContext, RenderError, RenderOptions, Renderer2D};
use forge_runtime::procedural::generate_hello_assets;
use forge_runtime::render_scene::{render_world_scene, DEFAULT_CLEAR_COLOR};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_headless_hello_scene_render_and_reference_comparison() {
    let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();

    // 1. Request adapter with fallback (software) support
    let render_options = RenderOptions {
        force_fallback_adapter: true,
        backends: wgpu::Backends::all(),
    };

    let context = match RenderContext::new(render_options) {
        Ok(ctx) => Arc::new(ctx),
        Err(RenderError::NoAdapterFound) => {
            eprintln!("SKIPPING GPU test: No compatible wgpu adapter found (hardware or software fallback).");
            return;
        }
        Err(e) => {
            eprintln!("SKIPPING GPU test: Failed to initialize GPU adapter: {e}");
            return;
        }
    };

    println!(
        "GPU adapter for headless test: name='{}', backend={:?}",
        context.adapter_info.name, context.adapter_info.backend
    );

    // 2. Discover workspace root and ensure procedural sprite assets exist
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = Path::new(manifest_dir).join("../..");

    let assets_dir = workspace_root.join("templates/hello/assets");
    generate_hello_assets(&assets_dir).expect("Failed to generate procedural assets");

    // 3. Load scene from workspace root
    let scene_path = workspace_root.join("templates/hello/scene.json");
    let scene = Scene::load_from_file(&scene_path).expect("Failed to load hello scene");
    let mut world = World::new();
    scene
        .spawn_into_world(&mut world)
        .expect("Failed to spawn scene into ECS world");

    // 4. Setup rendering pipeline (800x600)
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    let offscreen = OffscreenTarget::new(&context.device, WIDTH, HEIGHT)
        .expect("Failed to create offscreen render target");

    let mut asset_db = forge_assets::AssetDatabase::new(&assets_dir);
    asset_db
        .scan_and_index()
        .expect("Failed to scan asset database");
    world.insert_resource(asset_db.clone());

    let mut texture_store = TextureStore::new(&context).expect("Failed to create texture store");
    texture_store.set_database(asset_db);
    texture_store.set_base_dir(&workspace_root);

    let mut renderer = Renderer2D::new(context.clone());
    let mut game_loop = GameLoop::new();
    let frame_delta = Duration::from_secs_f64(1.0 / 60.0);

    // 5. Render 5 simulation frames
    for _ in 0..5 {
        game_loop.step(&mut world, frame_delta);
        render_world_scene(
            &world,
            &mut renderer,
            &mut texture_store,
            &context,
            &offscreen.view,
            offscreen.format,
            WIDTH as f32,
            HEIGHT as f32,
            Some(DEFAULT_CLEAR_COLOR),
        )
        .expect("Failed to render frame to offscreen target");
    }

    // 6. Read back rendered pixels
    let actual_pixels = offscreen
        .read_rgba(&context.device, &context.queue)
        .expect("Failed to read back offscreen pixels");

    assert_eq!(
        actual_pixels.len(),
        (WIDTH * HEIGHT * 4) as usize,
        "Readback buffer size mismatch"
    );

    // Save screenshot for CI artifact upload
    let root_screenshot = workspace_root.join("hello_screenshot.png");
    let _ = image::save_buffer(
        &root_screenshot,
        &actual_pixels,
        WIDTH,
        HEIGHT,
        image::ExtendedColorType::Rgba8,
    );

    // 7. Verify non-blank image: verify background and non-background pixels exist
    let mut non_bg_count = 0;
    let bg_r = (DEFAULT_CLEAR_COLOR[0] * 255.0).round() as u8;
    let bg_g = (DEFAULT_CLEAR_COLOR[1] * 255.0).round() as u8;
    let bg_b = (DEFAULT_CLEAR_COLOR[2] * 255.0).round() as u8;

    for chunk in actual_pixels.chunks_exact(4) {
        if chunk[0] != bg_r || chunk[1] != bg_g || chunk[2] != bg_b {
            non_bg_count += 1;
        }
    }

    assert!(
        non_bg_count > 500,
        "Image appears blank or lacks rendered sprites! Non-background pixel count: {}",
        non_bg_count
    );

    // 8. Compare against stored reference image
    let ref_path = workspace_root.join("crates/forge_runtime/tests/fixtures/hello_reference.png");

    if ref_path.exists() {
        let ref_img = image::open(&ref_path)
            .expect("Failed to open stored reference image")
            .to_rgba8();

        assert_eq!(
            ref_img.dimensions(),
            (WIDTH, HEIGHT),
            "Reference dimensions mismatch"
        );

        let ref_pixels = ref_img.into_raw();
        let mut total_sq_diff: f64 = 0.0;

        for (a, b) in actual_pixels.iter().zip(ref_pixels.iter()) {
            let diff = (*a as f64) - (*b as f64);
            total_sq_diff += diff * diff;
        }

        let mse = total_sq_diff / (actual_pixels.len() as f64);
        println!(
            "Headless hello scene comparison against reference: MSE = {:.4}",
            mse
        );

        // Verify MSE tolerance
        assert!(
            mse < 25.0,
            "Rendered scene diverges from reference image! MSE: {:.4}",
            mse
        );
    } else {
        eprintln!(
            "Reference image '{:?}' not found, skipping pixel comparison.",
            ref_path
        );
    }
}
