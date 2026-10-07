//! # Forge Assets
//!
//! Asset loading and GPU texture caching pipeline for 2D sprites.
//! Supports decoding PNG textures from memory or disk, caching them by asset ID,
//! and falling back to a visible magenta placeholder with tracing warnings when assets are missing.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod texture_store;

pub use error::AssetError;
pub use texture_store::TextureStore;
