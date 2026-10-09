//! Fixed-timestep input consumption and snapshot isolation.

use crate::action::ActionMap;
use crate::keyboard::KeyCode;
use crate::mouse::MouseButton;
use crate::state::InputState;
use forge_core::ecs::Resource;
use std::collections::HashSet;

/// Fixed timestep input snapshot resource consumed by fixed simulation systems.
///
/// Guaranteed behavior across variable sub-steps:
/// - If 0 fixed updates run in a frame, unconsumed transitions are held over.
/// - If 1 fixed update runs in a frame, `just_pressed` and `just_released` are active for that update.
/// - If >1 fixed updates run in a frame, `just_pressed` and `just_released` are consumed on the first tick
///   and cleared for subsequent ticks so they are NEVER duplicated across sub-steps.
#[derive(Resource, Debug, Clone, Default)]
pub struct FixedInput {
    // Keys
    pub pressed_keys: HashSet<KeyCode>,
    pub just_pressed_keys: HashSet<KeyCode>,
    pub just_released_keys: HashSet<KeyCode>,

    // Mouse
    pub pressed_mouse_buttons: HashSet<MouseButton>,
    pub just_pressed_mouse_buttons: HashSet<MouseButton>,
    pub just_released_mouse_buttons: HashSet<MouseButton>,

    // Actions
    pub pressed_actions: HashSet<String>,
    pub just_pressed_actions: HashSet<String>,
    pub just_released_actions: HashSet<String>,
    pub action_axes: std::collections::HashMap<String, f32>,

    /// Has any fixed tick run this frame yet?
    pub consumed_first_substep: bool,
}

impl FixedInput {
    /// Syncs and prepares the fixed input state for a new render frame.
    pub fn begin_frame(&mut self, state: &InputState, map: &ActionMap) {
        self.pressed_keys.clear();
        self.just_pressed_keys.clear();
        self.just_released_keys.clear();

        self.pressed_mouse_buttons.clear();
        self.just_pressed_mouse_buttons.clear();
        self.just_released_mouse_buttons.clear();

        self.pressed_actions.clear();
        self.just_pressed_actions.clear();
        self.just_released_actions.clear();
        self.action_axes.clear();

        self.consumed_first_substep = false;

        if !state.is_focused {
            return;
        }

        self.pressed_keys = state.pressed_keys.clone();
        self.just_pressed_keys = state.just_pressed_keys.clone();
        self.just_released_keys = state.just_released_keys.clone();

        self.pressed_mouse_buttons = state.pressed_mouse_buttons.clone();
        self.just_pressed_mouse_buttons = state.just_pressed_mouse_buttons.clone();
        self.just_released_mouse_buttons = state.just_released_mouse_buttons.clone();

        // Copy active actions and axes
        for action in map.actions.keys() {
            if map.action_pressed(action, state) {
                self.pressed_actions.insert(action.clone());
            }
            if map.action_just_pressed(action, state) {
                self.just_pressed_actions.insert(action.clone());
            }
            if map.action_just_released(action, state) {
                self.just_released_actions.insert(action.clone());
            }
            let axis = map.action_axis(action, state);
            if axis.abs() > 0.001 {
                self.action_axes.insert(action.clone(), axis);
            }
        }
    }

    /// Called at each fixed tick in `PreUpdate`.
    ///
    /// Consumes transient transitions on the first fixed sub-step and clears them for subsequent ticks.
    pub fn on_fixed_tick(&mut self) {
        if self.consumed_first_substep {
            // Subsequent sub-steps in the same frame do NOT repeat just_pressed/just_released
            self.just_pressed_keys.clear();
            self.just_released_keys.clear();
            self.just_pressed_mouse_buttons.clear();
            self.just_released_mouse_buttons.clear();
            self.just_pressed_actions.clear();
            self.just_released_actions.clear();
        } else {
            self.consumed_first_substep = true;
        }
    }

    /// Queries whether an action is pressed in the current fixed update.
    pub fn action_pressed(&self, action: &str) -> bool {
        self.pressed_actions.contains(action)
    }

    /// Queries whether an action was just pressed in this fixed update.
    pub fn action_just_pressed(&self, action: &str) -> bool {
        self.just_pressed_actions.contains(action)
    }

    /// Queries whether an action was just released in this fixed update.
    pub fn action_just_released(&self, action: &str) -> bool {
        self.just_released_actions.contains(action)
    }

    /// Queries an action's axis value in this fixed update.
    pub fn action_axis(&self, action: &str) -> f32 {
        self.action_axes.get(action).copied().unwrap_or(0.0)
    }
}
