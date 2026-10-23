//! # Forge Assets
//!
//! Asset loading and GPU texture caching pipeline for 2D sprites.
//! Supports decoding PNG textures from memory or disk, caching them by asset ID,
//! and falling back to a visible magenta placeholder with tracing warnings when assets are missing.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod animation;
pub mod database;
pub mod error;
pub mod spritesheet;
pub mod systems;
pub mod texture_store;
pub mod tiled;
pub mod tileset;
pub mod watcher;

pub use animation::{AnimationClip, AnimationEvent, LoopMode};
pub use database::{AssetDatabase, AssetEntry, AssetMeta, AssetType, META_EXTENSION};
pub use error::AssetError;
pub use spritesheet::{AtlasFrame, GridSlice, PixelRect, SpriteSheet, SpriteSheetLayout};
pub use systems::{animator_update_system, AnimationStore, SpriteSheetStore, TilesetStore};
pub use texture_store::TextureStore;
pub use tiled::{import_tiled_json, TiledLayerJson, TiledMapJson};
pub use tileset::{TileProperties, Tileset};
pub use watcher::{HotReloadEvent, HotReloadWatcher};
