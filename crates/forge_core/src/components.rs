//! Built-in ECS components for Forge.

use crate::ecs::{Component, Entity};
use serde::{Deserialize, Serialize};

/// Two-dimensional vector with x and y float coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    /// Horizontal coordinate.
    pub x: f32,
    /// Vertical coordinate.
    pub y: f32,
}

impl Vec2 {
    /// Vector at origin `(0.0, 0.0)`.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    /// Vector with both components set to `1.0`.
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };

    /// Creates a new 2D vector.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Default for Vec2 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from(arr: [f32; 2]) -> Self {
        Self::new(arr[0], arr[1])
    }
}

impl From<(f32, f32)> for Vec2 {
    fn from(tuple: (f32, f32)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

/// Stable string identifier for an entity that persists across scene saves and loads.
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub String);

impl EntityId {
    /// Creates a new stable entity identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Borrows the string identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Human-readable display name for an entity in the editor or hierarchy.
#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Name(pub String);

impl Name {
    /// Creates a new name component.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Borrows the name string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Name {
    fn default() -> Self {
        Self("Entity".to_string())
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Two-dimensional transform describing position, rotation in radians, and scale.
#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform2D {
    /// 2D translation position.
    pub position: Vec2,
    /// Rotation angle in radians.
    pub rotation: f32,
    /// 2D scale factor.
    pub scale: Vec2,
}

impl Transform2D {
    /// Identity transform at origin with zero rotation and unit scale.
    pub const IDENTITY: Self = Self {
        position: Vec2::ZERO,
        rotation: 0.0,
        scale: Vec2::ONE,
    };

    /// Creates a new transform from position, rotation (radians), and scale.
    pub fn new(position: Vec2, rotation: f32, scale: Vec2) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Creates a transform with the given position, zero rotation, and unit scale.
    pub fn from_position(position: Vec2) -> Self {
        Self {
            position,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }

    /// Creates a transform with the given position coordinates, zero rotation, and unit scale.
    pub fn from_xy(x: f32, y: f32) -> Self {
        Self::from_position(Vec2::new(x, y))
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// Parent relationship pointing to the parent `Entity`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parent(pub Entity);

impl Parent {
    /// Creates a new Parent reference.
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }

    /// Gets the parent entity.
    pub fn entity(&self) -> Entity {
        self.0
    }
}

/// Child relationships listing child `Entity` instances.
#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub struct Children(pub Vec<Entity>);

impl Children {
    /// Creates a new empty Children collection.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Adds a child entity.
    pub fn add(&mut self, entity: Entity) {
        if !self.0.contains(&entity) {
            self.0.push(entity);
        }
    }

    /// Removes a child entity.
    pub fn remove(&mut self, entity: Entity) {
        self.0.retain(|&e| e != entity);
    }

    /// Returns a slice of child entities.
    pub fn as_slice(&self) -> &[Entity] {
        &self.0
    }
}

/// 2D Sprite component defining rendered visual asset, size, color tint, frame index, UV bounds, and z-ordering.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sprite {
    /// Identifier for the sprite texture or spritesheet asset.
    pub asset_id: String,
    /// Dimensions of the sprite in world units.
    pub size: Vec2,
    /// RGBA color tint multiplier `[r, g, b, a]`.
    pub color_tint: [f32; 4],
    /// Depth ordering index (higher values rendered on top).
    pub z_order: i32,
    /// Optional frame index within a sprite sheet or atlas (0-indexed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<u32>,
    /// Optional explicit UV bounding box `[u_min, v_min, u_max, v_max]`.
    /// When None, defaults to full texture `[0.0, 0.0, 1.0, 1.0]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uv_rect: Option<[f32; 4]>,
}

impl Sprite {
    /// Creates a new sprite with the specified asset ID and size.
    pub fn new(asset_id: impl Into<String>, size: Vec2) -> Self {
        Self {
            asset_id: asset_id.into(),
            size,
            color_tint: [1.0, 1.0, 1.0, 1.0],
            z_order: 0,
            frame: None,
            uv_rect: None,
        }
    }

    /// Sets the color tint.
    pub fn with_color_tint(mut self, tint: [f32; 4]) -> Self {
        self.color_tint = tint;
        self
    }

    /// Sets the z-order.
    pub fn with_z_order(mut self, z_order: i32) -> Self {
        self.z_order = z_order;
        self
    }

    /// Sets the frame index.
    pub fn with_frame(mut self, frame: u32) -> Self {
        self.frame = Some(frame);
        self
    }

    /// Sets the UV rectangle bounds `[u_min, v_min, u_max, v_max]`.
    pub fn with_uv_rect(mut self, uv_rect: [f32; 4]) -> Self {
        self.uv_rect = Some(uv_rect);
        self
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            asset_id: String::new(),
            size: Vec2::new(100.0, 100.0),
            color_tint: [1.0, 1.0, 1.0, 1.0],
            z_order: 0,
            frame: None,
            uv_rect: None,
        }
    }
}

/// Playback state and timing for animated entities.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Animator {
    /// Asset ID of the `AnimationClip` to play.
    pub clip_asset_id: String,
    /// Whether the animation is currently advancing.
    pub playing: bool,
    /// Playback speed multiplier (1.0 = normal speed, -1.0 = reverse).
    pub speed: f32,
    /// Current normalized/accumulated playback time in seconds.
    #[serde(default)]
    pub elapsed_seconds: f32,
    /// Current active frame index within the animation clip.
    #[serde(default)]
    pub current_frame_index: usize,
}

impl Animator {
    /// Creates a new Animator playing the given animation clip at normal speed.
    pub fn new(clip_asset_id: impl Into<String>) -> Self {
        Self {
            clip_asset_id: clip_asset_id.into(),
            playing: true,
            speed: 1.0,
            elapsed_seconds: 0.0,
            current_frame_index: 0,
        }
    }

    /// Creates an animator in a paused state.
    pub fn paused(clip_asset_id: impl Into<String>) -> Self {
        Self {
            clip_asset_id: clip_asset_id.into(),
            playing: false,
            speed: 1.0,
            elapsed_seconds: 0.0,
            current_frame_index: 0,
        }
    }

    /// Sets playback speed.
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }
}

impl Default for Animator {
    fn default() -> Self {
        Self {
            clip_asset_id: String::new(),
            playing: true,
            speed: 1.0,
            elapsed_seconds: 0.0,
            current_frame_index: 0,
        }
    }
}

/// 2D Camera component defining view zoom and active viewport state.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Camera2D {
    /// Camera zoom level (1.0 is default unscaled zoom).
    pub zoom: f32,
    /// Whether this camera is actively rendering.
    pub is_active: bool,
}

impl Camera2D {
    /// Creates a new active 2D camera with default unit zoom.
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            is_active: true,
        }
    }

    /// Sets the camera zoom level.
    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom;
        self
    }
}

impl Default for Camera2D {
    fn default() -> Self {
        Self::new()
    }
}
