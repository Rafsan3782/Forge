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

    if args.headless {
        run_headless(&args)
    } else {
        run_window(args)
    }
}
