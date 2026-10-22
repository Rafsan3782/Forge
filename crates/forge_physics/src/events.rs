//! Physics events emitted to the `EventBus`.

use forge_core::ecs::Entity;
use forge_core::Vec2;

/// Collision event emitted when two colliders start, continue, or finish touching.
#[derive(Debug, Clone, PartialEq)]
pub enum CollisionEvent {
    /// Two colliders started touching.
    Started {
        /// First entity involved.
        entity1: Entity,
        /// Second entity involved.
        entity2: Entity,
        /// Normal vector pointing from entity1 to entity2 (if available).
        normal: Option<Vec2>,
    },
    /// Two colliders remain in contact during a physics step.
    Stay {
        /// First entity involved.
        entity1: Entity,
        /// Second entity involved.
        entity2: Entity,
    },
    /// Two colliders stopped touching.
    Stopped {
        /// First entity involved.
        entity1: Entity,
        /// Second entity involved.
        entity2: Entity,
    },
}

/// Trigger event emitted when a sensor collider overlaps with another collider.
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerEvent {
    /// Sensor collider started overlapping with a collider.
    Enter {
        /// Sensor entity.
        trigger_entity: Entity,
        /// Other entity that entered the trigger zone.
        other_entity: Entity,
    },
    /// Sensor collider stopped overlapping with a collider.
    Exit {
        /// Sensor entity.
        trigger_entity: Entity,
        /// Other entity that exited the trigger zone.
        other_entity: Entity,
    },
}
