//! # Forge Runtime
//!
//! Standalone game runtime library and binary supporting:
//! - Interactive windowed rendering via `winit` and `wgpu`
//! - Fixed-timestep simulation stepping from `forge_core`
//! - Headless offscreen frame rendering and PNG screenshot export
//! - Procedural asset generation for starter templates

#![warn(missing_docs)]

pub mod bridge_ops;
pub mod cli;
pub mod demo_input;
pub mod headless;
pub mod procedural;
pub mod render_scene;
pub mod window;

pub use cli::CliArgs;
pub use demo_input::{
    demo_audio_keyboard_system, demo_character_controller_system, demo_hero_movement_system,
};
pub use headless::run_headless;
pub use procedural::{generate_audio_demo_assets, generate_hello_assets, generate_tilemap_assets};
pub use render_scene::{render_world_scene, PhysicsDebugDraw, DEFAULT_CLEAR_COLOR};
pub use window::{run_window, WindowApp};
