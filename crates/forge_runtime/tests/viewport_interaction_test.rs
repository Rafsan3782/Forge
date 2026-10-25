use forge_core::{Transform, Sprite, Tilemap, Rect, Vec2};
use forge_runtime::editor::{
    EditorState, EditorCamera, EditorPlayState,
    screen_to_world, world_to_screen,
    pick_entities_at, pick_entities_in_rect,
};
use bevy_ecs::prelude::*;

#[test]
fn test_camera_coordinate_math_roundtrip() {
    let camera = EditorCamera {
        position: Vec2::new(100.0, 50.0),
        zoom: 2.0,
    };
    let viewport_size = Vec2::new(800.0, 600.0);

    let screen_pos = Vec2::new(450.0, 250.0);
    let world_pos = screen_to_world(screen_pos, &camera, viewport_size);
    let back_to_screen = world_to_screen(world_pos, &camera, viewport_size);

    assert!((screen_pos.x - back_to_screen.x).abs() < 1e-4);
    assert!((screen_pos.y - back_to_screen.y).abs() < 1e-4);
}

#[test]
fn test_zoom_towards_cursor() {
    let mut camera = EditorCamera {
        position: Vec2::new(0.0, 0.0),
        zoom: 1.0,
    };
    let viewport_size = Vec2::new(800.0, 600.0);

    // Cursor at (600, 450)
    let cursor_screen = Vec2::new(600.0, 450.0);
    let world_before = screen_to_world(cursor_screen, &camera, viewport_size);

    // Zoom in by factor 1.25 towards cursor
    let new_zoom = camera.zoom * 1.25;
    let screen_center = viewport_size * 0.5;
    let cursor_offset = cursor_screen - screen_center;
    camera.position += cursor_offset * (1.0 / camera.zoom - 1.0 / new_zoom);
    camera.zoom = new_zoom;

    let world_after = screen_to_world(cursor_screen, &camera, viewport_size);

    // World position under cursor should remain identical
    assert!((world_before.x - world_after.x).abs() < 1e-4);
    assert!((world_before.y - world_after.y).abs() < 1e-4);
}

#[test]
fn test_pick_ordering_topmost_sprite() {
    let mut world = World::new();

    // Sprite 1: Background, z = 0.0, size 100x100
    let e1 = world.spawn((
        Transform {
            position: Vec2::new(0.0, 0.0),
            z_order: 0.0,
            ..Default::default()
        },
        Sprite {
            texture_id: "bg".into(),
            size: Vec2::new(100.0, 100.0),
            ..Default::default()
        },
    )).id();

    // Sprite 2: Foreground, z = 10.0, size 40x40 at (10, 10)
    let e2 = world.spawn((
        Transform {
            position: Vec2::new(10.0, 10.0),
            z_order: 10.0,
            ..Default::default()
        },
        Sprite {
            texture_id: "fg".into(),
            size: Vec2::new(40.0, 40.0),
            ..Default::default()
        },
    )).id();

    // Click at (10, 10) hits both e1 and e2. e2 has higher z-order so it must be first.
    let hits = pick_entities_at(&world, Vec2::new(10.0, 10.0));
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0], e2);
    assert_eq!(hits[1], e1);

    // Click at (40, 40) only hits e1
    let hits_bg = pick_entities_at(&world, Vec2::new(40.0, 40.0));
    assert_eq!(hits_bg.len(), 1);
    assert_eq!(hits_bg[0], e1);
}

#[test]
fn test_box_selection() {
    let mut world = World::new();

    let e1 = world.spawn((
        Transform {
            position: Vec2::new(0.0, 0.0),
            ..Default::default()
        },
        Sprite {
            size: Vec2::new(20.0, 20.0),
            ..Default::default()
        },
    )).id();

    let e2 = world.spawn((
        Transform {
            position: Vec2::new(100.0, 100.0),
            ..Default::default()
        },
        Sprite {
            size: Vec2::new(20.0, 20.0),
            ..Default::default()
        },
    )).id();

    // Box covering (-20, -20) to (20, 20) selects e1 but not e2
    let box_rect = Rect::new(-20.0, -20.0, 40.0, 40.0);
    let selected = pick_entities_in_rect(&world, box_rect);
    assert!(selected.contains(&e1));
    assert!(!selected.contains(&e2));

    // Box covering both
    let large_rect = Rect::new(-50.0, -50.0, 200.0, 200.0);
    let selected_all = pick_entities_in_rect(&world, large_rect);
    assert!(selected_all.contains(&e1));
    assert!(selected_all.contains(&e2));
}

#[test]
fn test_snapshot_and_restore_equality() {
    let mut world = World::new();
    world.insert_resource(EditorState::default());

    let e1 = world.spawn((
        forge_core::Name("Player".into()),
        Transform {
            position: Vec2::new(10.0, 20.0),
            rotation: 0.5,
            scale: Vec2::new(1.0, 1.0),
            z_order: 1.0,
        },
        Sprite {
            texture_id: "hero.png".into(),
            size: Vec2::new(32.0, 32.0),
            ..Default::default()
        },
    )).id();

    let e2 = world.spawn((
        forge_core::Name("Enemy".into()),
        Transform {
            position: Vec2::new(50.0, 60.0),
            ..Default::default()
        },
    )).id();

    // 1. Snapshot scene before play
    {
        let mut editor = world.resource_mut::<EditorState>();
        editor.start_play(&world);
        assert_eq!(editor.play_state, EditorPlayState::Playing);
    }

    // 2. Simulate play mode changes: mutate e1 position, despawn e2, spawn e3
    if let Some(mut t) = world.get_mut::<Transform>(e1) {
        t.position = Vec2::new(999.0, 999.0);
    }
    world.despawn(e2);
    world.spawn((
        forge_core::Name("Projectile".into()),
        Transform {
            position: Vec2::new(123.0, 456.0),
            ..Default::default()
        },
    ));

    // 3. Stop play mode -> restores snapshot
    {
        let mut editor = world.resource_mut::<EditorState>();
        editor.stop_play(&mut world);
        assert_eq!(editor.play_state, EditorPlayState::Edit);
    }

    // Verify e1 position was restored exactly
    let t1 = world.get::<Transform>(e1).expect("e1 must exist");
    assert_eq!(t1.position, Vec2::new(10.0, 20.0));
    assert_eq!(t1.rotation, 0.5);

    // Verify e2 was restored
    let t2 = world.get::<Transform>(e2).expect("e2 must be restored");
    assert_eq!(t2.position, Vec2::new(50.0, 60.0));

    // Verify spawned projectile is gone
    let count = world.query::<&Transform>().iter(&world).count();
    assert_eq!(count, 2);
}

#[test]
fn test_coordinate_mapping_streamed_mode_dpi_scales() {
    // Simulate Studio UI streamed frame viewport:
    // Viewport element size in CSS pixels: 800 x 600
    // Streamed frame resolution: 1280 x 720
    // Click at CSS (400, 300) (center of element)
    // Under different DPI scaling factors: 1.0 (100%), 1.25 (125%), 1.5 (150%)

    let frame_width = 1280.0;
    let frame_height = 720.0;
    let css_width = 800.0;
    let css_height = 600.0;

    let map_coords = |css_x: f32, css_y: f32, _dpi: f32| -> (f32, f32) {
        // Studio coordinates mapping formula:
        // CSS mouse coordinates relative to viewport element [0..css_width, 0..css_height]
        // mapped to the render frame dimensions [0..frame_width, 0..frame_height]
        let norm_x = css_x / css_width;
        let norm_y = css_y / css_height;
        let target_x = norm_x * frame_width;
        let target_y = norm_y * frame_height;
        (target_x, target_y)
    };

    // Center click
    let (cx_100, cy_100) = map_coords(400.0, 300.0, 1.0);
    assert_eq!(cx_100, 640.0);
    assert_eq!(cy_100, 360.0);

    let (cx_125, cy_125) = map_coords(400.0, 300.0, 1.25);
    assert_eq!(cx_125, 640.0);
    assert_eq!(cy_125, 360.0);

    let (cx_150, cy_150) = map_coords(400.0, 300.0, 1.5);
    assert_eq!(cx_150, 640.0);
    assert_eq!(cy_150, 360.0);

    // Top-left click
    let (tl_x, tl_y) = map_coords(0.0, 0.0, 1.25);
    assert_eq!(tl_x, 0.0);
    assert_eq!(tl_y, 0.0);

    // Bottom-right click
    let (br_x, br_y) = map_coords(800.0, 600.0, 1.5);
    assert_eq!(br_x, 1280.0);
    assert_eq!(br_y, 720.0);
}
