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

    /// Length / magnitude of the vector.
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Normalized direction vector (or ZERO if zero length).
    pub fn normalize_or_zero(&self) -> Self {
        let len = self.length();
        if len > 1e-6 {
            Self::new(self.x / len, self.y / len)
        } else {
            Self::ZERO
        }
    }
}

impl Default for Vec2 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl std::ops::Div<f32> for Vec2 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
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

    /// Linearly interpolates between `self` and `next` with factor `alpha` in `[0.0, 1.0]`.
    pub fn lerp(&self, next: &Self, alpha: f32) -> Self {
        let t = alpha.clamp(0.0, 1.0);
        let pos = Vec2::new(
            self.position.x + (next.position.x - self.position.x) * t,
            self.position.y + (next.position.y - self.position.y) * t,
        );
        let rot = self.rotation + (next.rotation - self.rotation) * t;
        let scale = Vec2::new(
            self.scale.x + (next.scale.x - self.scale.x) * t,
            self.scale.y + (next.scale.y - self.scale.y) * t,
        );
        Self::new(pos, rot, scale)
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// Stores the previous tick's transform for smooth sub-frame render interpolation.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct PreviousTransform2D(pub Transform2D);

impl PreviousTransform2D {
    /// Creates a new PreviousTransform2D wrapper.
    pub fn new(transform: Transform2D) -> Self {
        Self(transform)
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

/// Simulation body type for a 2D rigid body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyType2D {
    /// Dynamic body simulated by physics forces and impulses.
    #[default]
    #[serde(alias = "Dynamic", alias = "DYNAMIC")]
    Dynamic,
    /// Kinematic body positioned manually via velocity or transform.
    #[serde(alias = "Kinematic", alias = "KINEMATIC")]
    Kinematic,
    /// Static immovable body with infinite mass.
    #[serde(alias = "Static", alias = "STATIC")]
    Static,
}

/// 2D Rigid body component controlling physics simulation characteristics.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RigidBody2D {
    /// Type of rigid body (dynamic, kinematic, static).
    #[serde(default)]
    pub body_type: BodyType2D,
    /// Multiplier applied to world gravity (1.0 = normal gravity).
    #[serde(default = "default_gravity_scale")]
    pub gravity_scale: f32,
    /// Whether rotation around the Z-axis is locked.
    #[serde(default)]
    pub lock_rotation: bool,
    /// Linear damping coefficient dissipating linear velocity.
    #[serde(default)]
    pub linear_damping: f32,
    /// Angular damping coefficient dissipating rotational velocity.
    #[serde(default)]
    pub angular_damping: f32,
    /// Linear velocity `[vx, vy]`.
    #[serde(default)]
    pub linear_velocity: Vec2,
    /// Angular velocity in radians per second.
    #[serde(default)]
    pub angular_velocity: f32,
    /// Whether the body can sleep when at rest.
    #[serde(default = "default_can_sleep")]
    pub can_sleep: bool,
}

fn default_gravity_scale() -> f32 {
    1.0
}

fn default_can_sleep() -> bool {
    true
}

impl RigidBody2D {
    /// Creates a dynamic rigid body with default gravity and no damping.
    pub fn dynamic() -> Self {
        Self {
            body_type: BodyType2D::Dynamic,
            gravity_scale: 1.0,
            lock_rotation: false,
            linear_damping: 0.0,
            angular_damping: 0.0,
            linear_velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            can_sleep: true,
        }
    }

    /// Creates a static immovable body.
    pub fn static_body() -> Self {
        Self {
            body_type: BodyType2D::Static,
            gravity_scale: 0.0,
            lock_rotation: true,
            linear_damping: 0.0,
            angular_damping: 0.0,
            linear_velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            can_sleep: true,
        }
    }

    /// Creates a kinematic body.
    pub fn kinematic() -> Self {
        Self {
            body_type: BodyType2D::Kinematic,
            gravity_scale: 0.0,
            lock_rotation: false,
            linear_damping: 0.0,
            angular_damping: 0.0,
            linear_velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            can_sleep: true,
        }
    }

    /// Sets the gravity scale.
    pub fn with_gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }

    /// Locks or unlocks rotation.
    pub fn with_lock_rotation(mut self, lock: bool) -> Self {
        self.lock_rotation = lock;
        self
    }

    /// Sets linear damping.
    pub fn with_linear_damping(mut self, damping: f32) -> Self {
        self.linear_damping = damping;
        self
    }
}

impl Default for RigidBody2D {
    fn default() -> Self {
        Self::dynamic()
    }
}

/// Geometric shape representation for 2D colliders.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColliderShape2D {
    /// Box / cuboid specified by half-extents `(half_width, half_height)`.
    #[serde(alias = "box", alias = "BOX")]
    Box {
        /// Half extents `(hx, hy)`.
        half_extents: Vec2,
    },
    /// Circle specified by radius.
    #[serde(alias = "circle", alias = "CIRCLE")]
    Circle {
        /// Circle radius.
        radius: f32,
    },
    /// Capsule specified by half-height along Y and radius.
    #[serde(alias = "capsule", alias = "CAPSULE")]
    Capsule {
        /// Radius of end caps.
        radius: f32,
        /// Half height of cylinder segment.
        half_height: f32,
    },
}

impl Default for ColliderShape2D {
    fn default() -> Self {
        Self::Box {
            half_extents: Vec2::new(16.0, 16.0),
        }
    }
}

/// 2D Collider component attached to an entity with or without a `RigidBody2D`.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Collider2D {
    /// Shape geometry.
    pub shape: ColliderShape2D,
    /// Center offset relative to entity position.
    #[serde(default)]
    pub offset: Vec2,
    /// Whether this collider is a sensor (detects overlap without physical reaction).
    #[serde(default)]
    pub is_sensor: bool,
    /// Bitmask for collision membership layers.
    #[serde(default = "default_collision_groups")]
    pub collision_groups: u32,
    /// Bitmask for collision interaction mask.
    #[serde(default = "default_collision_groups")]
    pub collision_mask: u32,
    /// Surface friction coefficient (0.0 = frictionless, 1.0 = rough).
    #[serde(default = "default_friction")]
    pub friction: f32,
    /// Restitution / bounciness coefficient in `[0.0, 1.0]`.
    #[serde(default)]
    pub restitution: f32,
    /// Material density (used to calculate body mass if not set).
    #[serde(default = "default_density")]
    pub density: f32,
}

fn default_collision_groups() -> u32 {
    0xFFFF_FFFF
}

fn default_friction() -> f32 {
    0.5
}

fn default_density() -> f32 {
    1.0
}

impl Collider2D {
    /// Creates a box collider with given half-extents.
    pub fn box_shape(half_width: f32, half_height: f32) -> Self {
        Self {
            shape: ColliderShape2D::Box {
                half_extents: Vec2::new(half_width, half_height),
            },
            offset: Vec2::ZERO,
            is_sensor: false,
            collision_groups: 0xFFFF_FFFF,
            collision_mask: 0xFFFF_FFFF,
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
        }
    }

    /// Creates a circle collider with given radius.
    pub fn circle(radius: f32) -> Self {
        Self {
            shape: ColliderShape2D::Circle { radius },
            offset: Vec2::ZERO,
            is_sensor: false,
            collision_groups: 0xFFFF_FFFF,
            collision_mask: 0xFFFF_FFFF,
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
        }
    }

    /// Creates a capsule collider with given radius and half-height.
    pub fn capsule(radius: f32, half_height: f32) -> Self {
        Self {
            shape: ColliderShape2D::Capsule {
                radius,
                half_height,
            },
            offset: Vec2::ZERO,
            is_sensor: false,
            collision_groups: 0xFFFF_FFFF,
            collision_mask: 0xFFFF_FFFF,
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
        }
    }

    /// Sets sensor flag.
    pub fn with_sensor(mut self, sensor: bool) -> Self {
        self.is_sensor = sensor;
        self
    }

    /// Sets offset.
    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    /// Sets friction.
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }

    /// Sets restitution.
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution;
        self
    }

    /// Sets collision layer and mask.
    pub fn with_layers(mut self, groups: u32, mask: u32) -> Self {
        self.collision_groups = groups;
        self.collision_mask = mask;
        self
    }
}

impl Default for Collider2D {
    fn default() -> Self {
        Self::box_shape(16.0, 16.0)
    }
}

/// Target audio channel / mixing bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioBus {
    /// Master bus affecting all playback.
    Master,
    /// Music bus with fade support.
    Music,
    /// Sound effects bus.
    Sfx,
}

impl Default for AudioBus {
    fn default() -> Self {
        Self::Sfx
    }
}

impl std::str::FromStr for AudioBus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "master" => Ok(Self::Master),
            "music" => Ok(Self::Music),
            "sfx" => Ok(Self::Sfx),
            other => Err(format!("Unknown audio bus '{}'", other)),
        }
    }
}

/// Audio playback source component.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioSource {
    /// Asset ID or path of the audio file in the asset database.
    pub asset: String,
    /// Playback volume (default 1.0).
    #[serde(default = "default_audio_source_volume")]
    pub volume: f32,
    /// Pitch multiplier (default 1.0).
    #[serde(default = "default_audio_source_pitch")]
    pub pitch: f32,
    /// Whether playback loops indefinitely.
    #[serde(default)]
    pub r#loop: bool,
    /// Routing bus (Master, Music, Sfx).
    #[serde(default)]
    pub bus: AudioBus,
    /// Automatically play sound on scene start.
    #[serde(default = "default_true")]
    pub play_on_start: bool,
    /// Optional 2D spatial pan and distance falloff relative to the camera AudioListener.
    #[serde(default)]
    pub spatial: bool,
    /// Maximum distance for spatial falloff (sound is inaudible beyond this).
    #[serde(default = "default_spatial_max_distance")]
    pub max_distance: f32,
    /// Spatial distance falloff exponent or factor.
    #[serde(default = "default_spatial_falloff")]
    pub falloff: f32,
}

fn default_audio_source_volume() -> f32 {
    1.0
}

fn default_audio_source_pitch() -> f32 {
    1.0
}

fn default_true() -> bool {
    true
}

fn default_spatial_max_distance() -> f32 {
    1000.0
}

fn default_spatial_falloff() -> f32 {
    1.0
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            asset: String::new(),
            volume: 1.0,
            pitch: 1.0,
            r#loop: false,
            bus: AudioBus::Sfx,
            play_on_start: true,
            spatial: false,
            max_distance: 1000.0,
            falloff: 1.0,
        }
    }
}

impl AudioSource {
    /// Creates a new audio source with given asset ID or path.
    pub fn new(asset: impl Into<String>) -> Self {
        Self {
            asset: asset.into(),
            ..Default::default()
        }
    }

    /// Sets the playback bus.
    pub fn with_bus(mut self, bus: AudioBus) -> Self {
        self.bus = bus;
        self
    }

    /// Sets looping flag.
    pub fn with_loop(mut self, r#loop: bool) -> Self {
        self.r#loop = r#loop;
        self
    }

    /// Sets volume.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    /// Sets spatial audio properties.
    pub fn with_spatial(mut self, max_distance: f32, falloff: f32) -> Self {
        self.spatial = true;
        self.max_distance = max_distance;
        self.falloff = falloff;
        self
    }
}

/// Audio listener component attached to the active camera or player entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AudioListener {
    /// Whether this listener is currently active.
    #[serde(default = "default_true")]
    pub is_active: bool,
}

impl Default for AudioListener {
    fn default() -> Self {
        Self { is_active: true }
    }
}

/// Script component referencing a TypeScript script file executed via QuickJS.
#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Script {
    /// Relative or project path to the `.ts` script file.
    pub path: String,
    /// Whether the script is currently enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Script {
    /// Creates a new `Script` component from a path.
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            enabled: true,
        }
    }

    /// Disables execution for this script.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Enables execution for this script.
    pub fn enable(&mut self) {
        self.enabled = true;
    }
}
