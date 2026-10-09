//! Unit tests for action mapping, input state transitions, and fixed timestep isolation.

use crate::{
    fixed_input_tick_system, ActionDefinition, ActionMap, FixedInput, GamepadAxis, GamepadButton,
    InputBinding, InputRingBuffer, InputState, KeyCode, MouseButton,
};
use forge_core::ecs::{Schedule, World};
use forge_core::{Camera2D, Transform2D, Vec2};

#[test]
fn test_keyboard_state_transitions() {
    let mut state = InputState::new();
    let mut buffer = InputRingBuffer::new(100);

    assert!(!state.is_key_pressed(KeyCode::Space));
    assert!(!state.is_key_just_pressed(KeyCode::Space));
    assert!(!state.is_key_just_released(KeyCode::Space));

    // Press space
    state.feed_key_press(KeyCode::Space, Some(&mut buffer));
    assert!(state.is_key_pressed(KeyCode::Space));
    assert!(state.is_key_just_pressed(KeyCode::Space));
    assert!(!state.is_key_just_released(KeyCode::Space));
    assert_eq!(buffer.len(), 1);

    // Advance to next frame: still held, but no longer just_pressed
    state.start_new_frame();
    assert!(state.is_key_pressed(KeyCode::Space));
    assert!(!state.is_key_just_pressed(KeyCode::Space));
    assert!(!state.is_key_just_released(KeyCode::Space));

    // Release space
    state.feed_key_release(KeyCode::Space, Some(&mut buffer));
    assert!(!state.is_key_pressed(KeyCode::Space));
    assert!(!state.is_key_just_pressed(KeyCode::Space));
    assert!(state.is_key_just_released(KeyCode::Space));
    assert_eq!(buffer.len(), 2);

    // Next frame: cleared
    state.start_new_frame();
    assert!(!state.is_key_pressed(KeyCode::Space));
    assert!(!state.is_key_just_pressed(KeyCode::Space));
    assert!(!state.is_key_just_released(KeyCode::Space));
}

#[test]
fn test_unfocused_window_ignores_input() {
    let mut state = InputState::new();
    state.feed_key_press(KeyCode::ArrowRight, None);
    state.feed_mouse_button_press(MouseButton::Left, None);
    state.feed_gamepad_button_press(GamepadButton::South, None);
    state.feed_gamepad_axis(GamepadAxis::LeftStickX, 0.8);

    assert!(state.is_key_pressed(KeyCode::ArrowRight));
    assert!(state.is_mouse_button_pressed(MouseButton::Left));
    assert!(state.is_gamepad_button_pressed(GamepadButton::South));
    assert_eq!(state.gamepad_axis(GamepadAxis::LeftStickX), 0.8);

    // Unfocus window
    state.set_focused(false);
    assert!(!state.is_key_pressed(KeyCode::ArrowRight));
    assert!(!state.is_mouse_button_pressed(MouseButton::Left));
    assert!(!state.is_gamepad_button_pressed(GamepadButton::South));
    assert_eq!(state.gamepad_axis(GamepadAxis::LeftStickX), 0.0);

    // Refocus
    state.set_focused(true);
    assert!(state.is_key_pressed(KeyCode::ArrowRight));
    assert!(state.is_mouse_button_pressed(MouseButton::Left));
}

#[test]
fn test_action_mapping_digital_and_axes() {
    let mut map = ActionMap::new();
    map.actions.insert(
        "jump".to_string(),
        ActionDefinition {
            description: "Jump".to_string(),
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
    map.actions.insert(
        "move_x".to_string(),
        ActionDefinition {
            description: "Horizontal axis".to_string(),
            bindings: vec![
                InputBinding::DualKeys {
                    negative: "ArrowLeft".to_string(),
                    positive: "ArrowRight".to_string(),
                },
                InputBinding::GamepadAxis {
                    axis: "LeftStickX".to_string(),
                    scale: 1.0,
                },
            ],
        },
    );

    let mut state = InputState::new();
    assert!(!map.action_pressed("jump", &state));
    assert_eq!(map.action_axis("move_x", &state), 0.0);

    // Press right arrow
    state.feed_key_press(KeyCode::ArrowRight, None);
    assert_eq!(map.action_axis("move_x", &state), 1.0);

    // Also press left arrow -> cancellation
    state.feed_key_press(KeyCode::ArrowLeft, None);
    assert_eq!(map.action_axis("move_x", &state), 0.0);

    state.feed_key_release(KeyCode::ArrowRight, None);
    assert_eq!(map.action_axis("move_x", &state), -1.0);

    // Gamepad button for jump
    state.feed_gamepad_button_press(GamepadButton::South, None);
    assert!(map.action_pressed("jump", &state));
    assert!(map.action_just_pressed("jump", &state));
}

#[test]
fn test_mouse_world_projection() {
    let mut state = InputState::new();
    state.feed_mouse_move(Vec2::new(400.0, 300.0), None); // Screen center for 800x600

    let camera = Camera2D::new();
    let cam_trans = Transform2D::from_position(Vec2::new(100.0, 50.0));

    // Screen center should map exactly to camera position
    let world_pos = state.mouse_world_position(&camera, &cam_trans, 800.0, 600.0);
    assert!((world_pos.x - 100.0).abs() < 1e-4);
    assert!((world_pos.y - 50.0).abs() < 1e-4);

    // Top-right corner of screen (800, 0)
    state.feed_mouse_move(Vec2::new(800.0, 0.0), None);
    let world_tr = state.mouse_world_position(&camera, &cam_trans, 800.0, 600.0);
    assert!((world_tr.x - (100.0 + 400.0)).abs() < 1e-4);
    assert!((world_tr.y - (50.0 + 300.0)).abs() < 1e-4);
}

#[test]
fn test_fixed_timestep_isolation_multiple_substeps() {
    let mut world = World::new();
    let map = ActionMap::default_template();
    let mut state = InputState::new();
    let mut fixed = FixedInput::default();

    let mut schedule = Schedule::default();
    schedule.add_systems(fixed_input_tick_system);

    // User presses Space on frame 1
    state.feed_key_press(KeyCode::Space, None);
    fixed.begin_frame(&state, &map);

    world.insert_resource(fixed);

    // Simulation runs 3 fixed sub-steps in this frame (e.g. catch-up after lag)
    // Sub-step 1:
    schedule.run(&mut world);
    {
        let f = world.resource::<FixedInput>();
        assert!(f.action_pressed("jump"));
        assert!(
            f.action_just_pressed("jump"),
            "First substep must receive just_pressed"
        );
    }

    // Sub-step 2:
    schedule.run(&mut world);
    {
        let f = world.resource::<FixedInput>();
        assert!(f.action_pressed("jump"));
        assert!(
            !f.action_just_pressed("jump"),
            "Second substep MUST NOT duplicate just_pressed"
        );
    }

    // Sub-step 3:
    schedule.run(&mut world);
    {
        let f = world.resource::<FixedInput>();
        assert!(f.action_pressed("jump"));
        assert!(
            !f.action_just_pressed("jump"),
            "Third substep MUST NOT duplicate just_pressed"
        );
    }
}

#[test]
fn test_simulated_input_holds_for_n_frames() {
    let mut state = InputState::new();
    state.simulate_key(KeyCode::KeyA, true, 2);

    assert!(state.is_key_pressed(KeyCode::KeyA));
    assert!(state.is_key_just_pressed(KeyCode::KeyA));

    // Frame 1
    state.start_new_frame();
    assert!(state.is_key_pressed(KeyCode::KeyA));
    assert!(!state.is_key_just_pressed(KeyCode::KeyA));

    // Frame 2
    state.start_new_frame();
    assert!(!state.is_key_pressed(KeyCode::KeyA));
    assert!(state.is_key_just_released(KeyCode::KeyA));

    // Frame 3
    state.start_new_frame();
    assert!(!state.is_key_pressed(KeyCode::KeyA));
    assert!(!state.is_key_just_released(KeyCode::KeyA));
}
