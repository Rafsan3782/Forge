//! Gilrs gamepad event pump updating `InputState`.

use crate::buffer::InputRingBuffer;
use crate::gamepad::{GamepadAxis, GamepadButton};
use crate::state::InputState;
use gilrs::{Event, EventType, Gilrs};

/// Adapter polling gilrs events and updating gamepad state.
pub struct GilrsAdapter {
    gilrs: Option<Gilrs>,
}

impl GilrsAdapter {
    /// Creates a new gilrs adapter, gracefully falling back if no audio/gamepad backend is present.
    pub fn new() -> Self {
        let gilrs = match Gilrs::new() {
            Ok(g) => Some(g),
            Err(e) => {
                tracing::debug!("Failed to initialize gilrs gamepad subsystem: {}", e);
                None
            }
        };
        Self { gilrs }
    }

    /// Polls pending gamepad events and feeds them into `InputState`.
    pub fn poll_events(
        &mut self,
        state: &mut InputState,
        mut buffer: Option<&mut InputRingBuffer>,
    ) {
        let gilrs = match self.gilrs.as_mut() {
            Some(g) => g,
            None => return,
        };

        while let Some(Event { event, .. }) = gilrs.next_event() {
            match event {
                EventType::ButtonPressed(btn, _) => {
                    let button = GamepadButton::from_gilrs(btn);
                    state.feed_gamepad_button_press(button, buffer.as_deref_mut());
                }
                EventType::ButtonReleased(btn, _) => {
                    let button = GamepadButton::from_gilrs(btn);
                    state.feed_gamepad_button_release(button, buffer.as_deref_mut());
                }
                EventType::AxisChanged(ax, val, _) => {
                    let axis = GamepadAxis::from_gilrs(ax);
                    state.feed_gamepad_axis(axis, val);
                }
                _ => {}
            }
        }
    }
}

impl Default for GilrsAdapter {
    fn default() -> Self {
        Self::new()
    }
}
