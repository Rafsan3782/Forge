//! System functions for input synchronization with fixed timestep simulation.

use crate::action::ActionMap;
use crate::fixed::FixedInput;
use crate::state::InputState;
use forge_core::ecs::ResMut;

/// PreUpdate system running inside each fixed simulation tick to advance `FixedInput`.
pub fn fixed_input_tick_system(mut fixed: ResMut<FixedInput>) {
    fixed.on_fixed_tick();
}

/// Helper system updating `FixedInput` before the game loop advances for the frame.
pub fn sync_frame_input(state: &InputState, map: &ActionMap, fixed: &mut FixedInput) {
    fixed.begin_frame(state, map);
}

/// Synchronizes `InputState` and `ActionMap` into `FixedInput` directly from `World`.
pub fn sync_world_frame_input(world: &mut forge_core::ecs::World) {
    world.resource_scope(|world, mut fixed: forge_core::ecs::Mut<FixedInput>| {
        if let (Some(state), Some(map)) = (
            world.get_resource::<InputState>(),
            world.get_resource::<ActionMap>(),
        ) {
            fixed.begin_frame(state, map);
        }
    });
}
