//! # Forge Render
//!
//! 2D rendering subsystem built on WGPU, featuring orthographic Camera2D projection,
//! batched textured quads with z-ordering, color tinting, alpha blending, WGSL shaders,
//! and support for both window surface and offscreen render targets.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use wgpu;

pub mod batch;
pub mod camera;
pub mod context;
pub mod error;
pub mod renderer;
pub mod target;
pub mod texture;
pub mod vertex;

pub use batch::{DrawBatch, QueuedSprite, SpriteBatcher};
pub use camera::CameraUniform;
pub use context::{RenderContext, RenderOptions};
pub use error::RenderError;
pub use renderer::Renderer2D;
pub use target::OffscreenTarget;
pub use texture::{create_default_sampler, create_texture_bind_group_layout, GpuTexture};
pub use vertex::{generate_sprite_quad, SpriteVertex};
