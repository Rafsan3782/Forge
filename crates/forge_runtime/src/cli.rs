//! Command-line argument definitions for the Forge runtime binary.

use clap::Parser;

/// Standalone game runtime binary for the Forge engine.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "forge_runtime",
    author = "Forge Engine Authors",
    version = "0.1.0",
    about = "Forge standalone game engine runtime supporting interactive windows and headless rendering"
)]
pub struct CliArgs {
    /// Path to the JSON scene file to load
    #[arg(short, long, default_value = "templates/hello/scene.json")]
    pub scene: String,

    /// Run offscreen headless rendering without creating a window
    #[arg(long, default_value_t = false)]
    pub headless: bool,

    /// Number of fixed simulation frames to render in headless mode
    #[arg(long, default_value_t = 1)]
    pub frames: u32,

    /// Output path to save the final rendered frame screenshot (PNG format)
    #[arg(long)]
    pub screenshot: Option<String>,

    /// Force usage of software/fallback adapter (e.g. WARP on Windows)
    #[arg(long, default_value_t = false)]
    pub force_fallback: bool,

    /// Viewport width in pixels
    #[arg(long, default_value_t = 800)]
    pub width: u32,

    /// Viewport height in pixels
    #[arg(long, default_value_t = 600)]
    pub height: u32,

    /// Optional explicit authentication token for the bridge WebSocket server
    #[arg(long)]
    pub bridge_token: Option<String>,

    /// Disable starting the embedded bridge WebSocket server
    #[arg(long, default_value_t = false)]
    pub no_bridge: bool,

    /// Keep headless mode running continuously until project.stop is received
    #[arg(long, default_value_t = false)]
    pub stay_open: bool,

    /// Win32 parent window handle (HWND) to embed the runtime window as a child
    #[arg(long)]
    pub embed_parent: Option<String>,

    /// Run demo input controller for the hello scene (arrow keys move hero sprite)
    #[arg(long, default_value_t = false)]
    pub demo_input: bool,

    /// Optional path to input_map.json (defaults to input_map.json next to scene or default)
    #[arg(long)]
    pub input_map: Option<String>,
}
