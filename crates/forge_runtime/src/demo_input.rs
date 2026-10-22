//! Demo movement controller for the Hello Forge scene.
//!
//! When `--demo-input` is passed, updates the Hero entity's `Transform2D`
//! using input actions (`move_x`, `move_y`) or direct arrow keys.

use forge_core::ecs::{Query, Res};
use forge_core::{Name, Transform2D};
use forge_input::FixedInput;

/// Movement speed in units per second (assuming 60 updates/sec = ~3.33 units/tick).
const HERO_SPEED: f32 = 200.0;
const FIXED_TIMESTEP_SEC: f32 = 1.0 / 60.0;

/// System that queries the Hero entity and updates its position based on `FixedInput`.
pub fn demo_hero_movement_system(
    input: Res<FixedInput>,
    mut query: Query<(&Name, &mut Transform2D)>,
) {
    let mut move_x = input.action_axis("move_x");
    let mut move_y = input.action_axis("move_y");

    // Fallback: direct arrow keys if not bound or actions evaluate to 0
    if move_x.abs() < 0.001 {
        if input
            .pressed_keys
            .contains(&forge_input::KeyCode::ArrowRight)
        {
            move_x += 1.0;
        }
        if input
            .pressed_keys
            .contains(&forge_input::KeyCode::ArrowLeft)
        {
            move_x -= 1.0;
        }
    }
    if move_y.abs() < 0.001 {
        if input.pressed_keys.contains(&forge_input::KeyCode::ArrowUp) {
            move_y += 1.0;
        }
        if input
            .pressed_keys
            .contains(&forge_input::KeyCode::ArrowDown)
        {
            move_y -= 1.0;
        }
    }

    if move_x.abs() < 0.001 && move_y.abs() < 0.001 {
        return;
    }

    // Normalize diagonal movement
    let length = (move_x * move_x + move_y * move_y).sqrt();
    let (dir_x, dir_y) = if length > 1.0 {
        (move_x / length, move_y / length)
    } else {
        (move_x, move_y)
    };

    let dx = dir_x * HERO_SPEED * FIXED_TIMESTEP_SEC;
    let dy = dir_y * HERO_SPEED * FIXED_TIMESTEP_SEC;

    for (name, mut transform) in query.iter_mut() {
        if name.as_str() == "Hero" {
            transform.position.x += dx;
            transform.position.y += dy;
        }
    }
}

/// System that drives any entity with `CharacterController2D` using arrow keys and jump (Space).
pub fn demo_character_controller_system(
    input: Res<FixedInput>,
    mut query: Query<(
        &mut forge_physics::CharacterController2D,
        &mut forge_core::Transform2D,
    )>,
) {
    let mut move_x = input.action_axis("move_x");
    if move_x.abs() < 0.001 {
        if input
            .pressed_keys
            .contains(&forge_input::KeyCode::ArrowRight)
        {
            move_x += 1.0;
        }
        if input
            .pressed_keys
            .contains(&forge_input::KeyCode::ArrowLeft)
        {
            move_x -= 1.0;
        }
    }

    let jump_pressed = input.action_just_pressed("jump")
        || input
            .just_pressed_keys
            .contains(&forge_input::KeyCode::Space);

    let speed = 250.0;
    let dt = 1.0 / 60.0;

    for (mut controller, _) in query.iter_mut() {
        let trans_x = move_x * speed * dt;
        let trans_y = if controller.is_grounded {
            if jump_pressed {
                350.0 * dt // Initial jump impulse
            } else {
                -9.8 * dt // Ground snap force
            }
        } else {
            // In air: fall with gravity
            -300.0 * dt
        };

        controller.translation = forge_core::Vec2::new(trans_x, trans_y);
    }
}
