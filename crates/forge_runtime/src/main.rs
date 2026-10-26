//! Standalone executable entry point for forge_runtime.

use clap::Parser;
use forge_runtime::cli::CliArgs;
use forge_runtime::headless::run_headless;
use forge_runtime::procedural::generate_hello_assets;
use forge_runtime::window::run_window;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = CliArgs::parse();

    // Ensure hello template procedural assets exist if missing
    let template_assets_dir = Path::new("templates/hello/assets");
    if !template_assets_dir.exists() || !template_assets_dir.join("hero.png").exists() {
        if let Err(e) = generate_hello_assets(template_assets_dir) {
            tracing::warn!("Failed to auto-generate hello template assets: {}", e);
        }
    }

    // Ensure tilemap-demo template procedural assets exist if missing
    let tilemap_assets_dir = Path::new("templates/tilemap-demo/assets");
    if !tilemap_assets_dir.exists() || !tilemap_assets_dir.join("tileset.png").exists() {
        if let Err(e) = forge_runtime::procedural::generate_tilemap_assets(tilemap_assets_dir) {
            tracing::warn!("Failed to auto-generate tilemap demo assets: {}", e);
        }
    }

    // Ensure audio-demo template procedural assets exist if missing
    let audio_assets_dir = Path::new("templates/audio-demo/assets");
    if !audio_assets_dir.exists() || !audio_assets_dir.join("tone1.wav").exists() {
        if let Err(e) = forge_runtime::procedural::generate_audio_demo_assets(audio_assets_dir) {
            tracing::warn!("Failed to auto-generate audio demo assets: {}", e);
        }
    }

    // Ensure platformer template procedural assets exist if missing
    let platformer_assets_dir = Path::new("templates/platformer/assets");
    if !platformer_assets_dir.exists() || !platformer_assets_dir.join("player_sheet.png").exists() {
        if let Err(e) = forge_runtime::procedural::generate_platformer_assets(platformer_assets_dir)
        {
            tracing::warn!("Failed to auto-generate platformer assets: {}", e);
        }
    }

    if args.headless {
        run_headless(&args)
    } else {
        run_window(args)
    }
}
