//! Input abstraction, action mapping, and device polling for Forge Engine.

pub mod action;
pub mod buffer;
pub mod fixed;
pub mod gamepad;
pub mod gilrs_adapter;
pub mod keyboard;
pub mod mouse;
pub mod state;
pub mod systems;
pub mod winit_adapter;

pub use action::{ActionDefinition, ActionMap, InputBinding};
pub use buffer::{InputRingBuffer, RecordedInputEvent};
pub use fixed::FixedInput;
pub use gamepad::{GamepadAxis, GamepadButton};
pub use gilrs_adapter::GilrsAdapter;
pub use keyboard::KeyCode;
pub use mouse::MouseButton;
pub use state::InputState;
pub use systems::{fixed_input_tick_system, sync_frame_input, sync_world_frame_input};
pub use winit_adapter::pump_winit_event;

#[cfg(test)]
mod tests;
