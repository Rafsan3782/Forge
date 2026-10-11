//! Tests for animation clip timing, loop modes, and events.

use forge_assets::{AnimationClip, AnimationStore, LoopMode};
use forge_core::{Animator, EventBus, GameLoop, Sprite, Time, World};
use std::time::Duration;

#[test]
fn test_animation_clip_sampling() {
    let clip = AnimationClip::new("walk", vec![10, 11, 12, 13], 4.0) // 4 fps = 0.25s per frame
        .with_loop_mode(LoopMode::Loop);

    assert_eq!(clip.duration_seconds(), 1.0);

    // t = 0.0s -> frame index 0, frame 10
    let (idx0, frame0) = clip.sample(0.0);
    assert_eq!(idx0, 0);
    assert_eq!(frame0, 10);

    // t = 0.3s -> frame index 1, frame 11
    let (idx1, frame1) = clip.sample(0.3);
    assert_eq!(idx1, 1);
    assert_eq!(frame1, 11);

    // t = 0.8s -> frame index 3, frame 13
    let (idx3, frame3) = clip.sample(0.8);
    assert_eq!(idx3, 3);
    assert_eq!(frame3, 13);

    // t = 1.25s -> wraps to frame index 1, frame 11
    let (idx_wrap, frame_wrap) = clip.sample(1.25);
    assert_eq!(idx_wrap, 1);
    assert_eq!(frame_wrap, 11);
}

#[test]
fn test_animation_once_and_pingpong() {
    // Once mode clamps at final frame
    let clip_once = AnimationClip::new("die", vec![0, 1, 2], 2.0).with_loop_mode(LoopMode::Once);

    let (idx, frame) = clip_once.sample(5.0);
    assert_eq!(idx, 2);
    assert_eq!(frame, 2);

    // PingPong mode alternates
    let clip_pp = AnimationClip::new("bob", vec![0, 1, 2], 2.0) // 1.5s forward, 1.5s back
        .with_loop_mode(LoopMode::PingPong);

    let (idx_fwd, _) = clip_pp.sample(0.5);
    assert_eq!(idx_fwd, 1);

    // At 1.6s, should be going backward
    let (idx_back, _) = clip_pp.sample(2.0);
    assert_eq!(idx_back, 1);
}

#[test]
fn test_animator_system_advancement_and_events() {
    let mut world = World::new();

    let clip = AnimationClip::new("hero_slash", vec![0, 1, 2], 10.0) // 0.1s per frame
        .with_event(1, "slash_hit", Some("heavy".to_string()));

    let mut anim_store = AnimationStore::new();
    anim_store.insert("clip_slash", clip);

    let event_bus = EventBus::new();

    world.insert_resource(Time::new(60.0));
    world.insert_resource(anim_store);
    world.insert_resource(event_bus);

    let entity = world
        .spawn((
            Animator::new("clip_slash"),
            Sprite::new("dummy_texture", forge_core::Vec2::new(32.0, 32.0)),
        ))
        .id();

    let mut game_loop = GameLoop::new();
    game_loop.add_update_system(forge_assets::animator_update_system);

    // Step 7 ticks at 60 Hz = ~0.116s (advances into frame index 1)
    for _ in 0..7 {
        game_loop.step(&mut world, Duration::from_secs_f64(1.0 / 60.0));
    }

    let sprite = world.get::<Sprite>(entity).unwrap();
    assert_eq!(sprite.frame, Some(1));

    let bus = world.get_resource::<EventBus>().unwrap();
    let events = bus.read::<forge_assets::AnimationEvent>();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].name, "slash_hit");
    assert_eq!(events[0].payload.as_deref(), Some("heavy"));
}
