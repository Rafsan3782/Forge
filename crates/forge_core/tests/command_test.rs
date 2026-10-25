use forge_core::{CommandAuthor, CommandStack, SceneCommand, World};
use std::collections::HashMap;

#[test]
fn test_command_stack_entity_lifecycle() {
    let mut world = World::new();
    let mut stack = CommandStack::new();

    assert!(!stack.is_dirty());

    // 1. Create Entity Command
    let create_cmd = SceneCommand::CreateEntity {
        entity_id: "test_hero".to_string(),
        name: "Test Hero".to_string(),
        parent_id: None,
        components: None,
    };

    let id1 = stack
        .execute(&mut world, create_cmd, CommandAuthor::User)
        .expect("execute create");
    assert!(!id1.is_empty());
    assert!(stack.is_dirty());

    // Verify entity exists
    assert!(forge_core::find_entity_by_id(&world, "test_hero").is_some());

    // 2. Undo Create Entity
    let undone = stack.undo(&mut world).expect("undo");
    assert!(undone.is_some());
    assert!(forge_core::find_entity_by_id(&world, "test_hero").is_none());

    // 3. Redo Create Entity
    let redone = stack.redo(&mut world).expect("redo");
    assert!(redone.is_some());
    assert!(forge_core::find_entity_by_id(&world, "test_hero").is_some());

    // 4. Set Component Command
    let mut new_pos = HashMap::new();
    new_pos.insert("position".to_string(), "[10.0, 20.0]".to_string());
    let set_cmd = SceneCommand::SetComponent {
        entity_id: "test_hero".to_string(),
        component_name: "Transform2D".to_string(),
        new_data: new_pos,
        prev_data: None,
    };
    stack
        .execute(&mut world, set_cmd, CommandAuthor::Agent)
        .expect("set component");

    let history = stack.get_history();
    assert_eq!(history.undo_stack.len(), 2);
    assert_eq!(history.undo_stack[1].author, "agent");

    // 5. Jump back to first command
    let jumped = stack.jump_to(&mut world, &id1).expect("jump_to");
    assert!(jumped);
    assert_eq!(stack.get_history().undo_stack.len(), 1);
    assert_eq!(stack.get_history().redo_stack.len(), 1);

    // 6. Mark Clean & Save
    stack.mark_clean(Some("test_scene.json".to_string()));
    assert!(!stack.is_dirty());
}

#[test]
fn test_command_stack_tilemap_edits() {
    let mut world = World::new();
    let mut stack = CommandStack::new();

    // Create Tilemap entity
    let mut tm = forge_core::Tilemap::new("tileset_1", forge_core::Vec2::new(16.0, 16.0));
    tm.set_tile("main", 5, 5, 1);

    world.spawn((
        forge_core::EntityId::new("map_1"),
        forge_core::Name::new("Tilemap"),
        tm,
    ));

    // Edit tile from 1 to 42
    let tile_cmd = SceneCommand::TilemapSetTile {
        entity_id: "map_1".to_string(),
        layer: "main".to_string(),
        coord: [5, 5],
        new_tile_id: 42,
        prev_tile_id: 1,
    };

    stack
        .execute(&mut world, tile_cmd, CommandAuthor::User)
        .expect("execute tile edit");

    let entity = forge_core::find_entity_by_id(&world, "map_1").unwrap();
    assert_eq!(
        world
            .get::<forge_core::Tilemap>(entity)
            .unwrap()
            .get_tile("main", 5, 5),
        42
    );

    // Undo tile edit -> should restore 1
    stack.undo(&mut world).expect("undo");
    assert_eq!(
        world
            .get::<forge_core::Tilemap>(entity)
            .unwrap()
            .get_tile("main", 5, 5),
        1
    );

    // Redo tile edit -> should restore 42
    stack.redo(&mut world).expect("redo");
    assert_eq!(
        world
            .get::<forge_core::Tilemap>(entity)
            .unwrap()
            .get_tile("main", 5, 5),
        42
    );
}
