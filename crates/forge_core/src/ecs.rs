//! ECS abstractions and re-exports wrapping `bevy_ecs`.
//!
//! Other crates in the Forge workspace should never import `bevy_ecs` directly;
//! they should import types through `forge_core::ecs` or the crate root.

pub use bevy_ecs::bundle::Bundle;
pub use bevy_ecs::change_detection::{Mut, Ref};
pub use bevy_ecs::component::Component;
pub use bevy_ecs::entity::Entity;
pub use bevy_ecs::query::{Added, Changed, Or, QueryData, QueryFilter, With, Without};
pub use bevy_ecs::schedule::{IntoSystemConfigs, Schedule, SystemSet};
pub use bevy_ecs::system::{
    Commands, In, IntoSystem, NonSend, NonSendMut, Query, Res, ResMut, Resource,
};
pub use bevy_ecs::world::{Mut as WorldMut, World};
