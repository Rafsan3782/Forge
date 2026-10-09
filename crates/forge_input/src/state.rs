//! Core input state resource tracking keyboard, mouse, and gamepad.

use crate::buffer::InputRingBuffer;
use crate::gamepad::{GamepadAxis, GamepadButton};
use crate::keyboard::KeyCode;
use crate::mouse::MouseButton;
use forge_core::ecs::Resource;
use forge_core::{Camera2D, Transform2D, Vec2};
use std::collections::{HashMap, HashSet};

/// Input state resource stored in ECS World.
#[derive(Resource, Debug, Clone)]
pub struct InputState {
    // Window focus
    pub is_focused: bool,

    // Keyboard state
    pub(crate) pressed_keys: HashSet<KeyCode>,
    pub(crate) just_pressed_keys: HashSet<KeyCode>,
    pub(crate) just_released_keys: HashSet<KeyCode>,

    // Mouse state
    pub(crate) mouse_screen_pos: Vec2,
    pub(crate) mouse_wheel_delta: Vec2,
    pub(crate) pressed_mouse_buttons: HashSet<MouseButton>,
    pub(crate) just_pressed_mouse_buttons: HashSet<MouseButton>,
    pub(crate) just_released_mouse_buttons: HashSet<MouseButton>,

    // Gamepad state
    pub(crate) gamepad_deadzone: f32,
    pub(crate) pressed_gamepad_buttons: HashSet<GamepadButton>,
    pub(crate) just_pressed_gamepad_buttons: HashSet<GamepadButton>,
    pub(crate) just_released_gamepad_buttons: HashSet<GamepadButton>,
    pub(crate) gamepad_axes: HashMap<GamepadAxis, f32>,

    // Simulation / testing queue
    simulated_key_holds: HashMap<KeyCode, u32>,
    simulated_action_holds: HashMap<String, u32>,
    simulated_axis_holds: HashMap<String, (f32, u32)>,

    // Frame counter for telemetry
    current_frame: u64,
}

impl InputState {
    /// Creates a new default input state.
    pub fn new() -> Self {
        Self {
            is_focused: true,
            pressed_keys: HashSet::new(),
            just_pressed_keys: HashSet::new(),
            just_released_keys: HashSet::new(),

            mouse_screen_pos: Vec2::ZERO,
            mouse_wheel_delta: Vec2::ZERO,
            pressed_mouse_buttons: HashSet::new(),
            just_pressed_mouse_buttons: HashSet::new(),
            just_released_mouse_buttons: HashSet::new(),

            gamepad_deadzone: 0.15,
            pressed_gamepad_buttons: HashSet::new(),
            just_pressed_gamepad_buttons: HashSet::new(),
            just_released_gamepad_buttons: HashSet::new(),
            gamepad_axes: HashMap::new(),

            simulated_key_holds: HashMap::new(),
            simulated_action_holds: HashMap::new(),
            simulated_axis_holds: HashMap::new(),

            current_frame: 0,
        }
    }

    // -------------------------------------------------------------------------
    // Keyboard queries
    // -------------------------------------------------------------------------

    /// Returns true if the key is currently held down.
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        if !self.is_focused {
            return false;
        }
        self.pressed_keys.contains(&key) || self.simulated_key_holds.contains_key(&key)
    }

    /// Returns true if the key was pressed this frame.
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        if !self.is_focused {
            return false;
        }
        self.just_pressed_keys.contains(&key)
    }

    /// Returns true if the key was released this frame.
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        if !self.is_focused {
            return false;
        }
        self.just_released_keys.contains(&key)
    }

    // -------------------------------------------------------------------------
    // Mouse queries
    // -------------------------------------------------------------------------

    /// Returns the cursor position in screen/window coordinates (pixels, origin top-left).
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_screen_pos
    }

    /// Converts the current screen mouse position to world coordinates using the camera.
    ///
    /// Coordinate convention matching CameraUniform:
    /// Screen center corresponds to camera position.
    /// +X extends right, +Y extends up in world space.
    pub fn mouse_world_position(
        &self,
        camera: &Camera2D,
        camera_transform: &Transform2D,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Vec2 {
        let vw = if viewport_width.abs() < 1e-4 {
            800.0
        } else {
            viewport_width
        };
        let vh = if viewport_height.abs() < 1e-4 {
            600.0
        } else {
            viewport_height
        };

        // Normalize screen coordinates from [0..vw, 0..vh] to [-1..1, -1..1] (Cartesian Y up)
        let ndc_x = (self.mouse_screen_pos.x - vw * 0.5) / (vw * 0.5);
        let ndc_y = (vh * 0.5 - self.mouse_screen_pos.y) / (vh * 0.5);

        let zoom = if camera.zoom <= 0.0 { 1.0 } else { camera.zoom };
        let half_w = (vw * 0.5) / zoom;
        let half_h = (vh * 0.5) / zoom;

        // View space coordinates
        let vx = ndc_x * half_w;
        let vy = ndc_y * half_h;

        // Apply camera rotation and translation
        let cos = camera_transform.rotation.cos();
        let sin = camera_transform.rotation.sin();

        let world_x = camera_transform.position.x + (vx * cos - vy * sin);
        let world_y = camera_transform.position.y + (vx * sin + vy * cos);

        Vec2::new(world_x, world_y)
    }

    /// Returns true if the mouse button is currently held down.
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        if !self.is_focused {
            return false;
        }
        self.pressed_mouse_buttons.contains(&button)
    }

    /// Returns true if the mouse button was pressed this frame.
    pub fn is_mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        if !self.is_focused {
            return false;
        }
        self.just_pressed_mouse_buttons.contains(&button)
    }

    /// Returns true if the mouse button was released this frame.
    pub fn is_mouse_button_just_released(&self, button: MouseButton) -> bool {
        if !self.is_focused {
            return false;
        }
        self.just_released_mouse_buttons.contains(&button)
    }

    /// Returns the mouse wheel scroll delta for this frame.
    pub fn mouse_wheel(&self) -> Vec2 {
        if !self.is_focused {
            Vec2::ZERO
        } else {
            self.mouse_wheel_delta
        }
    }

    // -------------------------------------------------------------------------
    // Gamepad queries
    // -------------------------------------------------------------------------

    /// Returns true if the gamepad button is currently pressed.
    pub fn is_gamepad_button_pressed(&self, button: GamepadButton) -> bool {
        if !self.is_focused {
            return false;
        }
        self.pressed_gamepad_buttons.contains(&button)
    }

    /// Returns true if the gamepad button was pressed this frame.
    pub fn is_gamepad_button_just_pressed(&self, button: GamepadButton) -> bool {
        if !self.is_focused {
            return false;
        }
        self.just_pressed_gamepad_buttons.contains(&button)
    }

    /// Returns true if the gamepad button was released this frame.
    pub fn is_gamepad_button_just_released(&self, button: GamepadButton) -> bool {
        if !self.is_focused {
            return false;
        }
        self.just_released_gamepad_buttons.contains(&button)
    }

    /// Returns the value of a gamepad axis with deadzone filtering (-1.0 to 1.0).
    pub fn gamepad_axis(&self, axis: GamepadAxis) -> f32 {
        if !self.is_focused {
            return 0.0;
        }
        let raw = self.gamepad_axes.get(&axis).copied().unwrap_or(0.0);
        if raw.abs() < self.gamepad_deadzone {
            0.0
        } else {
            raw
        }
    }

    // -------------------------------------------------------------------------
    // Low-level event processing & simulation
    // -------------------------------------------------------------------------

    /// Updates window focus status.
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
        if !focused {
            // Clear transient states when losing focus
            self.just_pressed_keys.clear();
            self.just_released_keys.clear();
            self.just_pressed_mouse_buttons.clear();
            self.just_released_mouse_buttons.clear();
            self.just_pressed_gamepad_buttons.clear();
            self.just_released_gamepad_buttons.clear();
        }
    }

    /// Feeds a raw key press event.
    pub fn feed_key_press(&mut self, key: KeyCode, buffer: Option<&mut InputRingBuffer>) {
        if self.pressed_keys.insert(key) {
            self.just_pressed_keys.insert(key);
            if let Some(buf) = buffer {
                buf.record(self.current_frame, "key_down", format!("{:?}", key), 1.0);
            }
        }
    }

    /// Feeds a raw key release event.
    pub fn feed_key_release(&mut self, key: KeyCode, buffer: Option<&mut InputRingBuffer>) {
        if self.pressed_keys.remove(&key) {
            self.just_released_keys.insert(key);
            if let Some(buf) = buffer {
                buf.record(self.current_frame, "key_up", format!("{:?}", key), 0.0);
            }
        }
    }

    /// Feeds cursor movement.
    pub fn feed_mouse_move(&mut self, pos: Vec2, buffer: Option<&mut InputRingBuffer>) {
        self.mouse_screen_pos = pos;
        if let Some(buf) = buffer {
            buf.record(
                self.current_frame,
                "mouse_move",
                format!("{},{}", pos.x, pos.y),
                0.0,
            );
        }
    }

    /// Feeds mouse wheel scroll delta.
    pub fn feed_mouse_wheel(&mut self, delta: Vec2, buffer: Option<&mut InputRingBuffer>) {
        self.mouse_wheel_delta = delta;
        if let Some(buf) = buffer {
            buf.record(
                self.current_frame,
                "mouse_wheel",
                format!("{},{}", delta.x, delta.y),
                0.0,
            );
        }
    }

    /// Feeds mouse button press.
    pub fn feed_mouse_button_press(
        &mut self,
        button: MouseButton,
        buffer: Option<&mut InputRingBuffer>,
    ) {
        if self.pressed_mouse_buttons.insert(button) {
            self.just_pressed_mouse_buttons.insert(button);
            if let Some(buf) = buffer {
                buf.record(
                    self.current_frame,
                    "mouse_down",
                    format!("{:?}", button),
                    1.0,
                );
            }
        }
    }

    /// Feeds mouse button release.
    pub fn feed_mouse_button_release(
        &mut self,
        button: MouseButton,
        buffer: Option<&mut InputRingBuffer>,
    ) {
        if self.pressed_mouse_buttons.remove(&button) {
            self.just_released_mouse_buttons.insert(button);
            if let Some(buf) = buffer {
                buf.record(self.current_frame, "mouse_up", format!("{:?}", button), 0.0);
            }
        }
    }

    /// Feeds gamepad button press.
    pub fn feed_gamepad_button_press(
        &mut self,
        button: GamepadButton,
        buffer: Option<&mut InputRingBuffer>,
    ) {
        if self.pressed_gamepad_buttons.insert(button) {
            self.just_pressed_gamepad_buttons.insert(button);
            if let Some(buf) = buffer {
                buf.record(
                    self.current_frame,
                    "gamepad_button_down",
                    format!("{:?}", button),
                    1.0,
                );
            }
        }
    }

    /// Feeds gamepad button release.
    pub fn feed_gamepad_button_release(
        &mut self,
        button: GamepadButton,
        buffer: Option<&mut InputRingBuffer>,
    ) {
        if self.pressed_gamepad_buttons.remove(&button) {
            self.just_released_gamepad_buttons.insert(button);
            if let Some(buf) = buffer {
                buf.record(
                    self.current_frame,
                    "gamepad_button_up",
                    format!("{:?}", button),
                    0.0,
                );
            }
        }
    }

    /// Feeds gamepad axis value.
    pub fn feed_gamepad_axis(&mut self, axis: GamepadAxis, value: f32) {
        self.gamepad_axes.insert(axis, value);
    }

    /// Simulates a key for N frames.
    pub fn simulate_key(&mut self, key: KeyCode, pressed: bool, duration_frames: u32) {
        if pressed {
            self.just_pressed_keys.insert(key);
            self.pressed_keys.insert(key);
            self.simulated_key_holds.insert(key, duration_frames.max(1));
        } else {
            self.simulated_key_holds.remove(&key);
            self.pressed_keys.remove(&key);
            self.just_released_keys.insert(key);
        }
    }

    /// Simulates an action for N frames.
    pub fn simulate_action(
        &mut self,
        action: impl Into<String>,
        pressed: bool,
        duration_frames: u32,
    ) {
        let name = action.into();
        if pressed {
            self.simulated_action_holds
                .insert(name, duration_frames.max(1));
        } else {
            self.simulated_action_holds.remove(&name);
        }
    }

    /// Simulates an action axis value for N frames.
    pub fn simulate_axis(&mut self, action: impl Into<String>, value: f32, duration_frames: u32) {
        self.simulated_axis_holds
            .insert(action.into(), (value, duration_frames.max(1)));
    }

    /// Returns whether a simulated action is held.
    pub fn is_simulated_action_pressed(&self, action: &str) -> bool {
        self.simulated_action_holds.contains_key(action)
    }

    /// Returns any simulated axis value if active.
    pub fn simulated_axis_value(&self, action: &str) -> Option<f32> {
        self.simulated_axis_holds.get(action).map(|(v, _)| *v)
    }

    /// Advances frame state at the start of a new render frame.
    ///
    /// Clears `just_pressed` and `just_released` transition flags and decrements simulated hold counters.
    pub fn start_new_frame(&mut self) {
        self.current_frame += 1;
        self.just_pressed_keys.clear();
        self.just_released_keys.clear();
        self.just_pressed_mouse_buttons.clear();
        self.just_released_mouse_buttons.clear();
        self.just_pressed_gamepad_buttons.clear();
        self.just_released_gamepad_buttons.clear();
        self.mouse_wheel_delta = Vec2::ZERO;

        // Decrement simulated key holds
        let mut keys_to_remove = Vec::new();
        for (key, count) in &mut self.simulated_key_holds {
            if *count <= 1 {
                keys_to_remove.push(*key);
            } else {
                *count -= 1;
            }
        }
        for key in keys_to_remove {
            self.simulated_key_holds.remove(&key);
            self.pressed_keys.remove(&key);
            self.just_released_keys.insert(key);
        }

        // Decrement simulated action holds
        self.simulated_action_holds.retain(|_, count| {
            if *count <= 1 {
                false
            } else {
                *count -= 1;
                true
            }
        });

        // Decrement simulated axis holds
        self.simulated_axis_holds.retain(|_, (_, count)| {
            if *count <= 1 {
                false
            } else {
                *count -= 1;
                true
            }
        });
    }

    /// Returns the current simulation frame number.
    pub fn current_frame(&self) -> u64 {
        self.current_frame
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}
