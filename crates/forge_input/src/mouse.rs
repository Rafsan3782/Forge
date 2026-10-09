//! Mouse buttons and pointer representations.

use serde::{Deserialize, Serialize};

/// Mouse button identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    /// Left primary mouse button.
    Left,
    /// Right secondary mouse button.
    Right,
    /// Middle wheel click.
    Middle,
    /// Additional mouse button (e.g. forward, back).
    Other(u16),
}

impl MouseButton {
    /// Parses a string representation of a mouse button.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "left" | "mouseleft" | "button1" => Some(MouseButton::Left),
            "right" | "mouseright" | "button2" => Some(MouseButton::Right),
            "middle" | "mousemiddle" | "button3" => Some(MouseButton::Middle),
            _ => None,
        }
    }

    /// Converts from winit's MouseButton.
    pub fn from_winit(button: &winit::event::MouseButton) -> Self {
        match button {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Back => MouseButton::Other(1),
            winit::event::MouseButton::Forward => MouseButton::Other(2),
            winit::event::MouseButton::Other(val) => MouseButton::Other(*val),
        }
    }
}
