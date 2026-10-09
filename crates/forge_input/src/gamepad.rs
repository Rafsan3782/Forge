//! Gamepad buttons and axes mapping backed by `gilrs`.

use serde::{Deserialize, Serialize};

/// Standard gamepad buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadButton {
    South,
    East,
    North,
    West,
    C,
    Z,
    LeftTrigger,
    LeftTrigger2,
    RightTrigger,
    RightTrigger2,
    Select,
    Start,
    Mode,
    LeftThumb,
    RightThumb,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    Unknown,
}

impl GamepadButton {
    /// Parses a string representation of a gamepad button.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "south" | "a" | "cross" => Some(GamepadButton::South),
            "east" | "b" | "circle" => Some(GamepadButton::East),
            "north" | "x" | "triangle" => Some(GamepadButton::North),
            "west" | "y" | "square" => Some(GamepadButton::West),
            "lefttrigger" | "l1" | "lb" => Some(GamepadButton::LeftTrigger),
            "lefttrigger2" | "l2" | "lt" => Some(GamepadButton::LeftTrigger2),
            "righttrigger" | "r1" | "rb" => Some(GamepadButton::RightTrigger),
            "righttrigger2" | "r2" | "rt" => Some(GamepadButton::RightTrigger2),
            "select" | "back" => Some(GamepadButton::Select),
            "start" => Some(GamepadButton::Start),
            "mode" | "home" | "guide" => Some(GamepadButton::Mode),
            "leftthumb" | "l3" => Some(GamepadButton::LeftThumb),
            "rightthumb" | "r3" => Some(GamepadButton::RightThumb),
            "dpadup" | "up" => Some(GamepadButton::DPadUp),
            "dpaddown" | "down" => Some(GamepadButton::DPadDown),
            "dpadleft" | "left" => Some(GamepadButton::DPadLeft),
            "dpadright" | "right" => Some(GamepadButton::DPadRight),
            _ => None,
        }
    }

    /// Converts from gilrs::Button.
    pub fn from_gilrs(button: gilrs::Button) -> Self {
        match button {
            gilrs::Button::South => GamepadButton::South,
            gilrs::Button::East => GamepadButton::East,
            gilrs::Button::North => GamepadButton::North,
            gilrs::Button::West => GamepadButton::West,
            gilrs::Button::C => GamepadButton::C,
            gilrs::Button::Z => GamepadButton::Z,
            gilrs::Button::LeftTrigger => GamepadButton::LeftTrigger,
            gilrs::Button::LeftTrigger2 => GamepadButton::LeftTrigger2,
            gilrs::Button::RightTrigger => GamepadButton::RightTrigger,
            gilrs::Button::RightTrigger2 => GamepadButton::RightTrigger2,
            gilrs::Button::Select => GamepadButton::Select,
            gilrs::Button::Start => GamepadButton::Start,
            gilrs::Button::Mode => GamepadButton::Mode,
            gilrs::Button::LeftThumb => GamepadButton::LeftThumb,
            gilrs::Button::RightThumb => GamepadButton::RightThumb,
            gilrs::Button::DPadUp => GamepadButton::DPadUp,
            gilrs::Button::DPadDown => GamepadButton::DPadDown,
            gilrs::Button::DPadLeft => GamepadButton::DPadLeft,
            gilrs::Button::DPadRight => GamepadButton::DPadRight,
            gilrs::Button::Unknown => GamepadButton::Unknown,
        }
    }
}

/// Standard gamepad analog axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    LeftZ,
    RightStickX,
    RightStickY,
    RightZ,
    DPadX,
    DPadY,
    Unknown,
}

impl GamepadAxis {
    /// Parses a string representation of a gamepad axis.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "leftstickx" | "lx" => Some(GamepadAxis::LeftStickX),
            "leftsticky" | "ly" => Some(GamepadAxis::LeftStickY),
            "rightstickx" | "rx" => Some(GamepadAxis::RightStickX),
            "rightsticky" | "ry" => Some(GamepadAxis::RightStickY),
            "leftz" | "lt" => Some(GamepadAxis::LeftZ),
            "rightz" | "rt" => Some(GamepadAxis::RightZ),
            "dpadx" => Some(GamepadAxis::DPadX),
            "dpady" => Some(GamepadAxis::DPadY),
            _ => None,
        }
    }

    /// Converts from gilrs::Axis.
    pub fn from_gilrs(axis: gilrs::Axis) -> Self {
        match axis {
            gilrs::Axis::LeftStickX => GamepadAxis::LeftStickX,
            gilrs::Axis::LeftStickY => GamepadAxis::LeftStickY,
            gilrs::Axis::LeftZ => GamepadAxis::LeftZ,
            gilrs::Axis::RightStickX => GamepadAxis::RightStickX,
            gilrs::Axis::RightStickY => GamepadAxis::RightStickY,
            gilrs::Axis::RightZ => GamepadAxis::RightZ,
            gilrs::Axis::DPadX => GamepadAxis::DPadX,
            gilrs::Axis::DPadY => GamepadAxis::DPadY,
            gilrs::Axis::Unknown => GamepadAxis::Unknown,
        }
    }
}
