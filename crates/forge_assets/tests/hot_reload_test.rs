//! Test for hot reload watcher detecting changes and reloading textures.

use forge_assets::{HotReloadEvent, HotReloadWatcher, TextureStore};
use forge_render::{RenderContext, RenderOptions};
use image::{Rgba, RgbaImage};
use std::fs;
use std::sync::Arc;
use std::time::Duration;

fn create_test_png(path: &std::path::Path, r: u8, g: u8, b: u8) {
    let mut img = RgbaImage::new(16, 16);
    for pixel in img.pixels_mut() {
        *pixel = Rgba([r, g, b, 255]);
    }
    img.save(path).unwrap();
}

#[test]
fn test_hot_reload_texture_in_headless_mode() {
    let render_options = RenderOptions {
        force_fallback_adapter: true,
        backends: forge_render::wgpu::Backends::all(),
    };

    let context = match RenderContext::new(render_options) {
        Ok(ctx) => Arc::new(ctx),
        Err(_) => {
            eprintln!("SKIPPING GPU hot reload test: No compatible wgpu adapter found.");
            return;
        }
    };

    let temp_dir = std::env::temp_dir().join(format!("forge_test_reload_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).unwrap();

    let texture_path = temp_dir.join("live_texture.png");
    create_test_png(&texture_path, 255, 0, 0); // Red

    let mut watcher = HotReloadWatcher::new(&temp_dir)
        .unwrap()
        .with_debounce(Duration::from_millis(20));

    let mut store = TextureStore::new(&context).unwrap();
    store.set_base_dir(&temp_dir);

    // Initial load
    let initial_tex = store.get_or_load("live_texture.png", &context);
    assert_eq!(initial_tex.width, 16);

    // Modify texture on disk
    std::thread::sleep(Duration::from_millis(50));
    create_test_png(&texture_path, 0, 255, 0); // Green

    // Poll for the event in a loop to ensure file system watcher notification arrives
    let mut detected = false;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        std::thread::sleep(Duration::from_millis(50));
        let events = watcher.poll_events();
        for ev in events {
            if let HotReloadEvent::Texture(p) = ev {
                if p == texture_path
                    || p.file_name() == texture_path.file_name()
                    || std::fs::canonicalize(&p).ok() == std::fs::canonicalize(&texture_path).ok()
                {
                    detected = true;
                    break;
                }
            }
        }
        if detected {
            break;
        }
    }

    assert!(
        detected,
        "Expected HotReloadEvent::Texture for modified file"
    );

    // Perform reload on store
    let reloaded_tex = store.reload("live_texture.png", &context).unwrap();
    assert_eq!(reloaded_tex.width, 16);

    let _ = fs::remove_dir_all(&temp_dir);
}
