//! Keyboard key codes and state representations.

use serde::{Deserialize, Serialize};

/// Keyboard key representation across physical and logical codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    // Arrow keys
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,

    // Alpha keys
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,

    // Digits
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,

    // Special keys
    Space,
    Enter,
    Escape,
    Tab,
    Backspace,
    ShiftLeft,
    ShiftRight,
    ControlLeft,
    ControlRight,
    AltLeft,
    AltRight,

    // Other key fallback
    Other(u32),
}

impl KeyCode {
    /// Parses a string representation of a key (e.g. "ArrowUp", "Space", "KeyW", "w", "W").
    pub fn from_name(name: &str) -> Option<Self> {
        let trimmed = name.trim();
        match trimmed {
            "ArrowUp" | "Up" => Some(KeyCode::ArrowUp),
            "ArrowDown" | "Down" => Some(KeyCode::ArrowDown),
            "ArrowLeft" | "Left" => Some(KeyCode::ArrowLeft),
            "ArrowRight" | "Right" => Some(KeyCode::ArrowRight),
            "Space" | " " => Some(KeyCode::Space),
            "Enter" | "Return" => Some(KeyCode::Enter),
            "Escape" | "Esc" => Some(KeyCode::Escape),
            "Tab" => Some(KeyCode::Tab),
            "Backspace" => Some(KeyCode::Backspace),
            "ShiftLeft" | "LShift" | "Shift" => Some(KeyCode::ShiftLeft),
            "ShiftRight" | "RShift" => Some(KeyCode::ShiftRight),
            "ControlLeft" | "LControl" | "Ctrl" | "Control" => Some(KeyCode::ControlLeft),
            "ControlRight" | "RControl" => Some(KeyCode::ControlRight),
            "AltLeft" | "LAlt" | "Alt" => Some(KeyCode::AltLeft),
            "AltRight" | "RAlt" => Some(KeyCode::AltRight),
            // Alpha keys
            s if s.eq_ignore_ascii_case("KeyA") || s.eq_ignore_ascii_case("a") => {
                Some(KeyCode::KeyA)
            }
            s if s.eq_ignore_ascii_case("KeyB") || s.eq_ignore_ascii_case("b") => {
                Some(KeyCode::KeyB)
            }
            s if s.eq_ignore_ascii_case("KeyC") || s.eq_ignore_ascii_case("c") => {
                Some(KeyCode::KeyC)
            }
            s if s.eq_ignore_ascii_case("KeyD") || s.eq_ignore_ascii_case("d") => {
                Some(KeyCode::KeyD)
            }
            s if s.eq_ignore_ascii_case("KeyE") || s.eq_ignore_ascii_case("e") => {
                Some(KeyCode::KeyE)
            }
            s if s.eq_ignore_ascii_case("KeyF") || s.eq_ignore_ascii_case("f") => {
                Some(KeyCode::KeyF)
            }
            s if s.eq_ignore_ascii_case("KeyG") || s.eq_ignore_ascii_case("g") => {
                Some(KeyCode::KeyG)
            }
            s if s.eq_ignore_ascii_case("KeyH") || s.eq_ignore_ascii_case("h") => {
                Some(KeyCode::KeyH)
            }
            s if s.eq_ignore_ascii_case("KeyI") || s.eq_ignore_ascii_case("i") => {
                Some(KeyCode::KeyI)
            }
            s if s.eq_ignore_ascii_case("KeyJ") || s.eq_ignore_ascii_case("j") => {
                Some(KeyCode::KeyJ)
            }
            s if s.eq_ignore_ascii_case("KeyK") || s.eq_ignore_ascii_case("k") => {
                Some(KeyCode::KeyK)
            }
            s if s.eq_ignore_ascii_case("KeyL") || s.eq_ignore_ascii_case("l") => {
                Some(KeyCode::KeyL)
            }
            s if s.eq_ignore_ascii_case("KeyM") || s.eq_ignore_ascii_case("m") => {
                Some(KeyCode::KeyM)
            }
            s if s.eq_ignore_ascii_case("KeyN") || s.eq_ignore_ascii_case("n") => {
                Some(KeyCode::KeyN)
            }
            s if s.eq_ignore_ascii_case("KeyO") || s.eq_ignore_ascii_case("o") => {
                Some(KeyCode::KeyO)
            }
            s if s.eq_ignore_ascii_case("KeyP") || s.eq_ignore_ascii_case("p") => {
                Some(KeyCode::KeyP)
            }
            s if s.eq_ignore_ascii_case("KeyQ") || s.eq_ignore_ascii_case("q") => {
                Some(KeyCode::KeyQ)
            }
            s if s.eq_ignore_ascii_case("KeyR") || s.eq_ignore_ascii_case("r") => {
                Some(KeyCode::KeyR)
            }
            s if s.eq_ignore_ascii_case("KeyS") || s.eq_ignore_ascii_case("s") => {
                Some(KeyCode::KeyS)
            }
            s if s.eq_ignore_ascii_case("KeyT") || s.eq_ignore_ascii_case("t") => {
                Some(KeyCode::KeyT)
            }
            s if s.eq_ignore_ascii_case("KeyU") || s.eq_ignore_ascii_case("u") => {
                Some(KeyCode::KeyU)
            }
            s if s.eq_ignore_ascii_case("KeyV") || s.eq_ignore_ascii_case("v") => {
                Some(KeyCode::KeyV)
            }
            s if s.eq_ignore_ascii_case("KeyW") || s.eq_ignore_ascii_case("w") => {
                Some(KeyCode::KeyW)
            }
            s if s.eq_ignore_ascii_case("KeyX") || s.eq_ignore_ascii_case("x") => {
                Some(KeyCode::KeyX)
            }
            s if s.eq_ignore_ascii_case("KeyY") || s.eq_ignore_ascii_case("y") => {
                Some(KeyCode::KeyY)
            }
            s if s.eq_ignore_ascii_case("KeyZ") || s.eq_ignore_ascii_case("z") => {
                Some(KeyCode::KeyZ)
            }
            // Digits
            "Digit0" | "0" => Some(KeyCode::Digit0),
            "Digit1" | "1" => Some(KeyCode::Digit1),
            "Digit2" | "2" => Some(KeyCode::Digit2),
            "Digit3" | "3" => Some(KeyCode::Digit3),
            "Digit4" | "4" => Some(KeyCode::Digit4),
            "Digit5" | "5" => Some(KeyCode::Digit5),
            "Digit6" | "6" => Some(KeyCode::Digit6),
            "Digit7" | "7" => Some(KeyCode::Digit7),
            "Digit8" | "8" => Some(KeyCode::Digit8),
            "Digit9" | "9" => Some(KeyCode::Digit9),
            _ => None,
        }
    }

    /// Converts from winit's KeyCode if available.
    pub fn from_winit_key(key: &winit::keyboard::Key) -> Option<Self> {
        match key {
            winit::keyboard::Key::Named(named) => match named {
                winit::keyboard::NamedKey::ArrowUp => Some(KeyCode::ArrowUp),
                winit::keyboard::NamedKey::ArrowDown => Some(KeyCode::ArrowDown),
                winit::keyboard::NamedKey::ArrowLeft => Some(KeyCode::ArrowLeft),
                winit::keyboard::NamedKey::ArrowRight => Some(KeyCode::ArrowRight),
                winit::keyboard::NamedKey::Space => Some(KeyCode::Space),
                winit::keyboard::NamedKey::Enter => Some(KeyCode::Enter),
                winit::keyboard::NamedKey::Escape => Some(KeyCode::Escape),
                winit::keyboard::NamedKey::Tab => Some(KeyCode::Tab),
                winit::keyboard::NamedKey::Backspace => Some(KeyCode::Backspace),
                winit::keyboard::NamedKey::Shift => Some(KeyCode::ShiftLeft),
                winit::keyboard::NamedKey::Control => Some(KeyCode::ControlLeft),
                winit::keyboard::NamedKey::Alt => Some(KeyCode::AltLeft),
                _ => None,
            },
            winit::keyboard::Key::Character(ch) => Self::from_name(ch.as_str()),
            _ => None,
        }
    }

    /// Converts from winit's physical key code.
    pub fn from_winit_physical(code: &winit::keyboard::KeyCode) -> Option<Self> {
        use winit::keyboard::KeyCode as WKeyCode;
        match code {
            WKeyCode::ArrowUp => Some(KeyCode::ArrowUp),
            WKeyCode::ArrowDown => Some(KeyCode::ArrowDown),
            WKeyCode::ArrowLeft => Some(KeyCode::ArrowLeft),
            WKeyCode::ArrowRight => Some(KeyCode::ArrowRight),
            WKeyCode::KeyA => Some(KeyCode::KeyA),
            WKeyCode::KeyB => Some(KeyCode::KeyB),
            WKeyCode::KeyC => Some(KeyCode::KeyC),
            WKeyCode::KeyD => Some(KeyCode::KeyD),
            WKeyCode::KeyE => Some(KeyCode::KeyE),
            WKeyCode::KeyF => Some(KeyCode::KeyF),
            WKeyCode::KeyG => Some(KeyCode::KeyG),
            WKeyCode::KeyH => Some(KeyCode::KeyH),
            WKeyCode::KeyI => Some(KeyCode::KeyI),
            WKeyCode::KeyJ => Some(KeyCode::KeyJ),
            WKeyCode::KeyK => Some(KeyCode::KeyK),
            WKeyCode::KeyL => Some(KeyCode::KeyL),
            WKeyCode::KeyM => Some(KeyCode::KeyM),
            WKeyCode::KeyN => Some(KeyCode::KeyN),
            WKeyCode::KeyO => Some(KeyCode::KeyO),
            WKeyCode::KeyP => Some(KeyCode::KeyP),
            WKeyCode::KeyQ => Some(KeyCode::KeyQ),
            WKeyCode::KeyR => Some(KeyCode::KeyR),
            WKeyCode::KeyS => Some(KeyCode::KeyS),
            WKeyCode::KeyT => Some(KeyCode::KeyT),
            WKeyCode::KeyU => Some(KeyCode::KeyU),
            WKeyCode::KeyV => Some(KeyCode::KeyV),
            WKeyCode::KeyW => Some(KeyCode::KeyW),
            WKeyCode::KeyX => Some(KeyCode::KeyX),
            WKeyCode::KeyY => Some(KeyCode::KeyY),
            WKeyCode::KeyZ => Some(KeyCode::KeyZ),
            WKeyCode::Digit0 => Some(KeyCode::Digit0),
            WKeyCode::Digit1 => Some(KeyCode::Digit1),
            WKeyCode::Digit2 => Some(KeyCode::Digit2),
            WKeyCode::Digit3 => Some(KeyCode::Digit3),
            WKeyCode::Digit4 => Some(KeyCode::Digit4),
            WKeyCode::Digit5 => Some(KeyCode::Digit5),
            WKeyCode::Digit6 => Some(KeyCode::Digit6),
            WKeyCode::Digit7 => Some(KeyCode::Digit7),
            WKeyCode::Digit8 => Some(KeyCode::Digit8),
            WKeyCode::Digit9 => Some(KeyCode::Digit9),
            WKeyCode::Space => Some(KeyCode::Space),
            WKeyCode::Enter => Some(KeyCode::Enter),
            WKeyCode::Escape => Some(KeyCode::Escape),
            WKeyCode::Tab => Some(KeyCode::Tab),
            WKeyCode::Backspace => Some(KeyCode::Backspace),
            WKeyCode::ShiftLeft => Some(KeyCode::ShiftLeft),
            WKeyCode::ShiftRight => Some(KeyCode::ShiftRight),
            WKeyCode::ControlLeft => Some(KeyCode::ControlLeft),
            WKeyCode::ControlRight => Some(KeyCode::ControlRight),
            WKeyCode::AltLeft => Some(KeyCode::AltLeft),
            WKeyCode::AltRight => Some(KeyCode::AltRight),
            _ => None,
        }
    }
}
