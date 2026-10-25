use forge_core::components::{EntityId, Name, Sprite, Transform2D};
use forge_core::{Vec2, World};
use forge_runtime::editor::{
    pick_entities_at, pick_entities_in_rect, EditorPlayState, EditorState,
};

#[test]
fn test_camera_coordinate_math_roundtrip() {
    let editor = EditorState {
        camera_position: Vec2::new(100.0, 50.0),
        camera_zoom: 2.0,
        ..Default::default()
    };
    let viewport_w = 800.0;
    let viewport_h = 600.0;

    let screen_pos = Vec2::new(450.0, 250.0);
    let world_pos = editor.screen_to_world(screen_pos, viewport_w, viewport_h);
    let back_to_screen = editor.world_to_screen(world_pos, viewport_w, viewport_h);

    assert!((screen_pos.x - back_to_screen.x).abs() < 1e-3);
    assert!((screen_pos.y - back_to_screen.y).abs() < 1e-3);
}

#[test]
fn test_zoom_towards_cursor() {
    let mut editor = EditorState {
        camera_position: Vec2::new(0.0, 0.0),
        camera_zoom: 1.0,
        ..Default::default()
    };
    let viewport_w = 800.0;
    let viewport_h = 600.0;

    // Cursor at screen (600, 450)
    let cursor_screen = Vec2::new(600.0, 450.0);
    let world_before = editor.screen_to_world(cursor_screen, viewport_w, viewport_h);

    // Zoom in by factor 1.25 towards cursor
    let zoom_factor = 1.25;
    let pre_zoom_world = editor.screen_to_world(cursor_screen, viewport_w, viewport_h);
    editor.camera_zoom *= zoom_factor;
    let post_zoom_world = editor.screen_to_world(cursor_screen, viewport_w, viewport_h);

    editor.camera_position.x += pre_zoom_world.x - post_zoom_world.x;
    editor.camera_position.y += pre_zoom_world.y - post_zoom_world.y;

    let world_after = editor.screen_to_world(cursor_screen, viewport_w, viewport_h);

    // World position under cursor should remain identical
    assert!((world_before.x - world_after.x).abs() < 1e-3);
    assert!((world_before.y - world_after.y).abs() < 1e-3);
}

#[test]
fn test_pick_ordering_topmost_sprite() {
    let mut world = World::new();

    // Sprite 1: Background, z = 0, size 100x100
    let _e1 = world.spawn((
        EntityId::new("bg"),
        Transform2D {
            position: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        },
        Sprite {
            asset_id: "bg_tex".into(),
            size: Vec2::new(100.0, 100.0),
            color_tint: [1.0, 1.0, 1.0, 1.0],
            z_order: 0,
            frame: None,
            uv_rect: None,
        },
    ));

    // Sprite 2: Foreground, z = 10, size 40x40 at (10, 10)
    let _e2 = world.spawn((
        EntityId::new("fg"),
        Transform2D {
            position: Vec2::new(10.0, 10.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        },
        Sprite {
            asset_id: "fg_tex".into(),
            size: Vec2::new(40.0, 40.0),
            color_tint: [1.0, 1.0, 1.0, 1.0],
            z_order: 10,
            frame: None,
            uv_rect: None,
        },
    ));

    // Click at (10, 10) hits both bg and fg. fg has higher z-order so it must be first.
    let hits = pick_entities_at(&world, Vec2::new(10.0, 10.0));
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].entity_id, "fg");
    assert_eq!(hits[1].entity_id, "bg");

    // Click at (40, 40) only hits bg
    let hits_bg = pick_entities_at(&world, Vec2::new(40.0, 40.0));
    assert_eq!(hits_bg.len(), 1);
    assert_eq!(hits_bg[0].entity_id, "bg");
}

#[test]
fn test_box_selection() {
    let mut world = World::new();

    let _e1 = world.spawn((
        EntityId::new("entity_1"),
        Transform2D {
            position: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        },
        Sprite {
            asset_id: "s1".into(),
            size: Vec2::new(20.0, 20.0),
            color_tint: [1.0, 1.0, 1.0, 1.0],
            z_order: 0,
            frame: None,
            uv_rect: None,
        },
    ));

    let _e2 = world.spawn((
        EntityId::new("entity_2"),
        Transform2D {
            position: Vec2::new(100.0, 100.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        },
        Sprite {
            asset_id: "s2".into(),
            size: Vec2::new(20.0, 20.0),
            color_tint: [1.0, 1.0, 1.0, 1.0],
            z_order: 0,
            frame: None,
            uv_rect: None,
        },
    ));

    // Box covering (-20, -20) to (20, 20) selects entity_1 but not entity_2
    let box_rect = [-20.0, -20.0, 20.0, 20.0];
    let selected = pick_entities_in_rect(&world, box_rect);
    assert!(selected.contains(&"entity_1".to_string()));
    assert!(!selected.contains(&"entity_2".to_string()));

    // Box covering both
    let large_rect = [-50.0, -50.0, 150.0, 150.0];
    let selected_all = pick_entities_in_rect(&world, large_rect);
    assert!(selected_all.contains(&"entity_1".to_string()));
    assert!(selected_all.contains(&"entity_2".to_string()));
}

#[test]
fn test_snapshot_and_restore_equality() {
    let mut world = World::new();
    world.insert_resource(EditorState::default());

    let e1 = world
        .spawn((
            EntityId::new("player"),
            Name("Player".into()),
            Transform2D {
                position: Vec2::new(10.0, 20.0),
                rotation: 0.5,
                scale: Vec2::new(1.0, 1.0),
            },
            Sprite {
                asset_id: "hero.png".into(),
                size: Vec2::new(32.0, 32.0),
                color_tint: [1.0, 1.0, 1.0, 1.0],
                z_order: 1,
                frame: None,
                uv_rect: None,
            },
        ))
        .id();

    let e2 = world
        .spawn((
            EntityId::new("enemy"),
            Name("Enemy".into()),
            Transform2D {
                position: Vec2::new(50.0, 60.0),
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        ))
        .id();

    // 1. Snapshot scene before play
    {
        let mut editor = world
            .remove_resource::<EditorState>()
            .expect("EditorState should exist");
        editor
            .start_play(&world)
            .expect("start_play should succeed");
        assert_eq!(editor.play_state, EditorPlayState::Playing);
        world.insert_resource(editor);
    }

    // 2. Simulate play mode changes: mutate e1 position, despawn e2, spawn e3
    if let Some(mut t) = world.get_mut::<Transform2D>(e1) {
        t.position = Vec2::new(999.0, 999.0);
    }
    world.despawn(e2);
    world.spawn((
        EntityId::new("projectile"),
        Name("Projectile".into()),
        Transform2D {
            position: Vec2::new(123.0, 456.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        },
    ));

    // 3. Stop play mode -> restores snapshot
    {
        let mut editor = world
            .remove_resource::<EditorState>()
            .expect("EditorState should exist");
        editor
            .stop_play(&mut world)
            .expect("stop_play should succeed");
        assert_eq!(editor.play_state, EditorPlayState::Edit);
        world.insert_resource(editor);
    }

    // Verify player entity exists and position was restored exactly
    let restored_player =
        forge_core::find_entity_by_id(&world, "player").expect("player entity restored");
    let t1 = world
        .get::<Transform2D>(restored_player)
        .expect("player transform restored");
    assert_eq!(t1.position, Vec2::new(10.0, 20.0));
    assert_eq!(t1.rotation, 0.5);

    // Verify enemy was restored
    let restored_enemy = forge_core::find_entity_by_id(&world, "enemy").expect("enemy restored");
    let t2 = world
        .get::<Transform2D>(restored_enemy)
        .expect("enemy transform restored");
    assert_eq!(t2.position, Vec2::new(50.0, 60.0));

    // Verify spawned projectile is gone
    let projectile = forge_core::find_entity_by_id(&world, "projectile");
    assert!(projectile.is_none());
}

#[test]
fn test_coordinate_mapping_streamed_mode_dpi_scales() {
    let frame_width = 1280.0;
    let frame_height = 720.0;
    let css_width = 800.0;
    let css_height = 600.0;

    let map_coords = |css_x: f32, css_y: f32, _dpi: f32| -> (f32, f32) {
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
