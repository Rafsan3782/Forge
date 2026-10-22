//! Physics query types for raycasting and spatial overlap checks.

use forge_core::ecs::Entity;
use forge_core::Vec2;

/// Output hit information from a physics query.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsHit {
    /// Entity with the collider that was hit.
    pub entity: Entity,
    /// World position coordinate where contact occurred.
    pub point: Vec2,
    /// Surface normal vector pointing outward from the hit surface.
    pub normal: Vec2,
    /// Distance along the ray or sweep cast.
    pub distance: f32,
    /// Fraction `t` along the ray path `[0.0, 1.0]`.
    pub fraction: f32,
}

/// Query filter options for restricting raycasts and overlaps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueryFilter {
    /// Mask matching `collider.collision_groups`.
    pub mask: u32,
    /// Entity to ignore in query (e.g. self).
    pub exclude_entity: Option<Entity>,
    /// Whether to hit sensor / trigger colliders (default false).
    pub query_sensors: bool,
}

impl Default for QueryFilter {
    fn default() -> Self {
        Self {
            mask: 0xFFFF_FFFF,
            exclude_entity: None,
            query_sensors: false,
        }
    }
}

impl QueryFilter {
    /// Creates a filter with all layers enabled.
    pub fn new() -> Self {
        Self::default()
    }

    /// Excludes an entity from query results.
    pub fn excluding(mut self, entity: Entity) -> Self {
        self.exclude_entity = Some(entity);
        self
    }

    /// Sets whether to include sensors.
    pub fn with_sensors(mut self, include_sensors: bool) -> Self {
        self.query_sensors = include_sensors;
        self
    }

    /// Sets mask.
    pub fn with_mask(mut self, mask: u32) -> Self {
        self.mask = mask;
        self
    }
}
