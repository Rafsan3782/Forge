//! Action mapping translating physical bindings to high-level game actions.

use crate::gamepad::{GamepadAxis, GamepadButton};
use crate::keyboard::KeyCode;
use crate::mouse::MouseButton;
use crate::state::InputState;
use forge_core::ecs::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A physical binding that triggers a digital button or analog axis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputBinding {
    /// Keyboard key binding.
    Key { code: String },
    /// Mouse button binding.
    MouseButton { button: String },
    /// Gamepad digital button binding.
    GamepadButton { button: String },
    /// Gamepad analog axis binding with multiplier (e.g. 1.0 or -1.0).
    GamepadAxis {
        axis: String,
        #[serde(default = "default_axis_scale")]
        scale: f32,
    },
    /// Digital axis binding combining negative and positive key/button inputs.
    DualKeys { negative: String, positive: String },
}

fn default_axis_scale() -> f32 {
    1.0
}

/// An action definition composed of multiple alternate input bindings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionDefinition {
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
    /// List of alternative bindings that trigger this action.
    pub bindings: Vec<InputBinding>,
}

/// Project action map configuration matching `input_map.json`.
#[derive(Resource, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionMap {
    /// Map of action name to definition.
    pub actions: HashMap<String, ActionDefinition>,
}

impl ActionMap {
    /// Creates a new empty action map.
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    /// Loads an action map from a JSON file.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let map = serde_json::from_str(&content)?;
        Ok(map)
    }

    /// Creates the standard default action map for starter game templates.
    pub fn default_template() -> Self {
        let mut actions = HashMap::new();

        actions.insert(
            "move_x".to_string(),
            ActionDefinition {
                description: "Horizontal movement (-1.0 left to +1.0 right)".to_string(),
                bindings: vec![
                    InputBinding::DualKeys {
                        negative: "ArrowLeft".to_string(),
                        positive: "ArrowRight".to_string(),
                    },
                    InputBinding::DualKeys {
                        negative: "KeyA".to_string(),
                        positive: "KeyD".to_string(),
                    },
                    InputBinding::GamepadAxis {
                        axis: "LeftStickX".to_string(),
                        scale: 1.0,
                    },
                    InputBinding::DualKeys {
                        negative: "DPadLeft".to_string(),
                        positive: "DPadRight".to_string(),
                    },
                ],
            },
        );

        actions.insert(
            "move_y".to_string(),
            ActionDefinition {
                description: "Vertical movement (-1.0 down to +1.0 up)".to_string(),
                bindings: vec![
                    InputBinding::DualKeys {
                        negative: "ArrowDown".to_string(),
                        positive: "ArrowUp".to_string(),
                    },
                    InputBinding::DualKeys {
                        negative: "KeyS".to_string(),
                        positive: "KeyW".to_string(),
                    },
                    InputBinding::GamepadAxis {
                        axis: "LeftStickY".to_string(),
                        scale: 1.0,
                    },
                    InputBinding::DualKeys {
                        negative: "DPadDown".to_string(),
                        positive: "DPadUp".to_string(),
                    },
                ],
            },
        );

        actions.insert(
            "jump".to_string(),
            ActionDefinition {
                description: "Jump action".to_string(),
                bindings: vec![
                    InputBinding::Key {
                        code: "Space".to_string(),
                    },
                    InputBinding::GamepadButton {
                        button: "South".to_string(),
                    },
                ],
            },
        );

        actions.insert(
            "interact".to_string(),
            ActionDefinition {
                description: "Interact with objects".to_string(),
                bindings: vec![
                    InputBinding::Key {
                        code: "KeyE".to_string(),
                    },
                    InputBinding::GamepadButton {
                        button: "West".to_string(),
                    },
                ],
            },
        );

        Self { actions }
    }

    /// Evaluates if an action is currently held down.
    pub fn action_pressed(&self, action: &str, state: &InputState) -> bool {
        if state.is_simulated_action_pressed(action) {
            return true;
        }

        let def = match self.actions.get(action) {
            Some(d) => d,
            None => return false,
        };

        for binding in &def.bindings {
            match binding {
                InputBinding::Key { code } => {
                    if let Some(key) = KeyCode::from_name(code) {
                        if state.is_key_pressed(key) {
                            return true;
                        }
                    }
                }
                InputBinding::MouseButton { button } => {
                    if let Some(btn) = MouseButton::from_name(button) {
                        if state.is_mouse_button_pressed(btn) {
                            return true;
                        }
                    }
                }
                InputBinding::GamepadButton { button } => {
                    if let Some(btn) = GamepadButton::from_name(button) {
                        if state.is_gamepad_button_pressed(btn) {
                            return true;
                        }
                    }
                }
                InputBinding::GamepadAxis { axis, scale } => {
                    if let Some(ax) = GamepadAxis::from_name(axis) {
                        let val = state.gamepad_axis(ax) * scale;
                        if val > 0.5 {
                            return true;
                        }
                    }
                }
                InputBinding::DualKeys { positive, negative } => {
                    if let Some(k) = KeyCode::from_name(positive) {
                        if state.is_key_pressed(k) {
                            return true;
                        }
                    }
                    if let Some(k) = KeyCode::from_name(negative) {
                        if state.is_key_pressed(k) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Evaluates if an action was just pressed this frame.
    pub fn action_just_pressed(&self, action: &str, state: &InputState) -> bool {
        let def = match self.actions.get(action) {
            Some(d) => d,
            None => return false,
        };

        for binding in &def.bindings {
            match binding {
                InputBinding::Key { code } => {
                    if let Some(key) = KeyCode::from_name(code) {
                        if state.is_key_just_pressed(key) {
                            return true;
                        }
                    }
                }
                InputBinding::MouseButton { button } => {
                    if let Some(btn) = MouseButton::from_name(button) {
                        if state.is_mouse_button_just_pressed(btn) {
                            return true;
                        }
                    }
                }
                InputBinding::GamepadButton { button } => {
                    if let Some(btn) = GamepadButton::from_name(button) {
                        if state.is_gamepad_button_just_pressed(btn) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }

        false
    }

    /// Evaluates if an action was just released this frame.
    pub fn action_just_released(&self, action: &str, state: &InputState) -> bool {
        let def = match self.actions.get(action) {
            Some(d) => d,
            None => return false,
        };

        for binding in &def.bindings {
            match binding {
                InputBinding::Key { code } => {
                    if let Some(key) = KeyCode::from_name(code) {
                        if state.is_key_just_released(key) {
                            return true;
                        }
                    }
                }
                InputBinding::MouseButton { button } => {
                    if let Some(btn) = MouseButton::from_name(button) {
                        if state.is_mouse_button_just_released(btn) {
                            return true;
                        }
                    }
                }
                InputBinding::GamepadButton { button } => {
                    if let Some(btn) = GamepadButton::from_name(button) {
                        if state.is_gamepad_button_just_released(btn) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }

        false
    }

    /// Returns the float value (-1.0 to 1.0) of an analog or dual-key axis action.
    pub fn action_axis(&self, action: &str, state: &InputState) -> f32 {
        if let Some(sim_val) = state.simulated_axis_value(action) {
            return sim_val.clamp(-1.0, 1.0);
        }

        let def = match self.actions.get(action) {
            Some(d) => d,
            None => return 0.0,
        };

        let mut value = 0.0f32;

        for binding in &def.bindings {
            match binding {
                InputBinding::DualKeys { negative, positive } => {
                    let mut pos = 0.0f32;
                    let mut neg = 0.0f32;

                    if let Some(k) = KeyCode::from_name(positive) {
                        if state.is_key_pressed(k) {
                            pos = 1.0;
                        }
                    }
                    if let Some(k) = KeyCode::from_name(negative) {
                        if state.is_key_pressed(k) {
                            neg = 1.0;
                        }
                    }

                    let diff = pos - neg;
                    if diff.abs() > value.abs() {
                        value = diff;
                    }
                }
                InputBinding::GamepadAxis { axis, scale } => {
                    if let Some(ax) = GamepadAxis::from_name(axis) {
                        let val = state.gamepad_axis(ax) * scale;
                        if val.abs() > value.abs() {
                            value = val;
                        }
                    }
                }
                InputBinding::Key { code } => {
                    if let Some(k) = KeyCode::from_name(code) {
                        if state.is_key_pressed(k) && value.abs() < 1.0 {
                            value = 1.0;
                        }
                    }
                }
                _ => {}
            }
        }

        value.clamp(-1.0, 1.0)
    }
}

impl Default for ActionMap {
    fn default() -> Self {
        Self::default_template()
    }
}
