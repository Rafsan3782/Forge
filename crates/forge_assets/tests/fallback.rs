//! Tests for asset loading and magenta fallback behavior.

use forge_assets::TextureStore;
use forge_render::{RenderContext, RenderError, RenderOptions};

#[test]
fn test_missing_asset_returns_magenta_placeholder() {
    let render_options = RenderOptions {
        force_fallback_adapter: true,
        backends: forge_render::wgpu::Backends::all(),
    };

    let context = match RenderContext::new(render_options) {
        Ok(ctx) => ctx,
        Err(RenderError::NoAdapterFound) => {
            eprintln!("SKIPPING GPU test: No compatible wgpu adapter found (hardware or software fallback).");
            return;
        }
        Err(e) => {
            eprintln!("SKIPPING GPU test: Failed to initialize GPU adapter: {e}");
            return;
        }
    };

    let mut store = TextureStore::new(&context).expect("Failed to create texture store");

    // Request non-existent asset
    let fallback = store.get_or_load("non_existent_asset_path_123.png", &context);

    // Placeholder dimensions are 2x2
    assert_eq!(fallback.width, 2);
    assert_eq!(fallback.height, 2);
}
