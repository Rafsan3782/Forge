//! Physics simulation and collision detection integration.

#![warn(missing_docs)]

pub mod controller;
pub mod debug;
pub mod events;
pub mod query;
pub mod systems;
pub mod world;

pub use controller::CharacterController2D;
pub use debug::{generate_collider_debug_lines, generate_contact_debug_lines, DebugLine};
pub use events::{CollisionEvent, TriggerEvent};
pub use query::{PhysicsHit, QueryFilter};
pub use systems::{physics_after_step_system, physics_before_step_system, physics_step_system};
pub use world::PhysicsWorld2D;

pub use rapier2d;
pub use rapier2d::na as nalgebra;
