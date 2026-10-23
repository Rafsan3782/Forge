//! # Forge Core
//!
//! Foundational engine systems, ECS abstractions wrapping `bevy_ecs`, built-in 2D components,
//! scene persistence model, fixed timestep game loop, typed event bus, and protocol scene operations.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod components;
pub mod ecs;
pub mod error;
pub mod events;
pub mod game_loop;
pub mod scene;
pub mod scene_ops;
pub mod tilemap;
pub mod time;

// Re-export primary ECS types so downstream crates never import bevy_ecs directly
pub use ecs::{Bundle, Commands, Component, Entity, Query, Res, ResMut, Resource, Schedule, World};

// Re-export built-in components and math primitives
pub use components::{
    Animator, AudioBus, AudioListener, AudioSource, BodyType2D, Camera2D, Children, Collider2D,
    ColliderShape2D, EntityId, Name, Parent, PreviousTransform2D, RigidBody2D, Script, Sprite,
    Transform2D, Vec2,
};
pub use tilemap::{format_row_string, parse_row_string, ChunkCoord, Tilemap, TilemapChunk};

// Re-export error types
pub use error::{CoreError, SceneError};

// Re-export event bus
pub use events::EventBus;

// Re-export time and game loop
pub use game_loop::{GameLoop, Stage};
pub use time::Time;

// Re-export scene model and serialization
pub use scene::{Scene, SceneEntity};

// Re-export scene operations and extension trait
pub use scene_ops::{
    build_entity_node, find_entity_by_id, scene_create_entity, scene_delete_entity, scene_get_tree,
    scene_set_component, WorldSceneExt,
};

#[cfg(test)]
mod tests;
