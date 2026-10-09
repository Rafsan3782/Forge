//! Winit window event pump translating OS events to `InputState`.

use crate::buffer::InputRingBuffer;
use crate::keyboard::KeyCode;
use crate::mouse::MouseButton;
use crate::state::InputState;
use forge_core::Vec2;
use winit::event::{ElementState, MouseScrollDelta, WindowEvent};

/// Pumps a winit `WindowEvent` into the `InputState` resource.
pub fn pump_winit_event(
    event: &WindowEvent,
    state: &mut InputState,
    buffer: Option<&mut InputRingBuffer>,
) {
    match event {
        WindowEvent::Focused(focused) => {
            state.set_focused(*focused);
        }
        WindowEvent::KeyboardInput { event, .. } => {
            // Prefer physical key code if recognized, otherwise logical key
            let key_code = match event.physical_key {
                winit::keyboard::PhysicalKey::Code(ref code) => KeyCode::from_winit_physical(code),
                _ => None,
            }
            .or_else(|| KeyCode::from_winit_key(&event.logical_key));

            if let Some(key) = key_code {
                match event.state {
                    ElementState::Pressed => {
                        state.feed_key_press(key, buffer);
                    }
                    ElementState::Released => {
                        state.feed_key_release(key, buffer);
                    }
                }
            }
        }
        WindowEvent::CursorMoved { position, .. } => {
            state.feed_mouse_move(Vec2::new(position.x as f32, position.y as f32), buffer);
        }
        WindowEvent::MouseInput {
            state: btn_state,
            button,
            ..
        } => {
            let mouse_button = MouseButton::from_winit(button);
            match btn_state {
                ElementState::Pressed => {
                    state.feed_mouse_button_press(mouse_button, buffer);
                }
                ElementState::Released => {
                    state.feed_mouse_button_release(mouse_button, buffer);
                }
            }
        }
        WindowEvent::MouseWheel { delta, .. } => {
            let (dx, dy) = match delta {
                MouseScrollDelta::LineDelta(x, y) => (*x, *y),
                MouseScrollDelta::PixelDelta(pos) => (pos.x as f32 * 0.1, pos.y as f32 * 0.1),
            };
            state.feed_mouse_wheel(Vec2::new(dx, dy), buffer);
        }
        _ => {}
    }
}
