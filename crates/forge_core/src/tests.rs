use crate::components::*;
use crate::ecs::*;
use crate::events::EventBus;
use crate::game_loop::GameLoop;
use crate::scene::Scene;
use crate::scene_ops::*;
use crate::time::Time;
use forge_protocol::{
    SceneCreateEntityParams, SceneDeleteEntityParams, SceneGetTreeParams, SceneSetComponentParams,
};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Component, Debug, PartialEq)]
struct CustomTestComp(u32);

#[test]
fn test_ecs_wrapping() {
    let mut world = World::new();
    let entity = world.spawn((CustomTestComp(42), Name::new("Custom"))).id();

    let comp = world.get::<CustomTestComp>(entity);
    assert_eq!(comp, Some(&CustomTestComp(42)));

    let name = world.get::<Name>(entity);
    assert_eq!(name.map(|n| n.as_str()), Some("Custom"));
}

#[test]
fn test_built_in_components() {
    let mut world = World::new();
    let parent_ent = world.spawn(Name::new("Parent")).id();

    let child_ent = world
        .spawn((
            Name::new("Child"),
            Transform2D::from_xy(10.0, 20.0),
            Sprite::new("sprites/coin.png", Vec2::new(32.0, 32.0))
                .with_color_tint([1.0, 0.5, 0.5, 1.0])
                .with_z_order(5),
            Camera2D::new().with_zoom(2.0),
            Parent::new(parent_ent),
        ))
        .id();

    let mut children = Children::new();
    children.add(child_ent);
    world.entity_mut(parent_ent).insert(children);

    assert_eq!(
        world.get::<Transform2D>(child_ent).unwrap().position.x,
        10.0
    );
    assert_eq!(world.get::<Sprite>(child_ent).unwrap().z_order, 5);
    assert_eq!(world.get::<Camera2D>(child_ent).unwrap().zoom, 2.0);
    assert_eq!(world.get::<Parent>(child_ent).unwrap().entity(), parent_ent);
    assert_eq!(
        world.get::<Children>(parent_ent).unwrap().as_slice(),
        &[child_ent]
    );
}

#[test]
fn test_scene_save_load_roundtrip_and_hierarchy() {
    let mut original_world = World::new();

    let root = original_world
        .spawn((
            EntityId::new("root_node"),
            Name::new("Root"),
            Transform2D::from_xy(0.0, 0.0),
        ))
        .id();

    let child = original_world
        .spawn((
            EntityId::new("child_node"),
            Name::new("Child"),
            Transform2D::from_xy(10.0, 15.0),
            Sprite::new("icon.png", Vec2::new(64.0, 64.0)),
            Parent::new(root),
        ))
        .id();

    let mut root_children = Children::new();
    root_children.add(child);
    original_world.entity_mut(root).insert(root_children);

    let grandchild = original_world
        .spawn((
            EntityId::new("grandchild_node"),
            Name::new("Grandchild"),
            Parent::new(child),
        ))
        .id();

    let mut child_children = Children::new();
    child_children.add(grandchild);
    original_world.entity_mut(child).insert(child_children);

    // Save world to Scene
    let scene =
        Scene::from_world(&original_world, "Test Hierarchy").expect("from_world should succeed");
    let json = scene.to_json_string().expect("serialize should succeed");

    // Load Scene back
    let loaded_scene = Scene::from_json_str(&json).expect("deserialize should succeed");
    assert_eq!(scene, loaded_scene);

    let mut restored_world = World::new();
    let entity_map = loaded_scene
        .spawn_into_world(&mut restored_world)
        .expect("spawn_into_world should succeed");

    assert_eq!(entity_map.len(), 3);
    let restored_root = entity_map["root_node"];
    let restored_child = entity_map["child_node"];
    let restored_grandchild = entity_map["grandchild_node"];

    // Check components
    assert_eq!(
        restored_world.get::<Name>(restored_root).unwrap().as_str(),
        "Root"
    );
    assert_eq!(
        restored_world
            .get::<Sprite>(restored_child)
            .unwrap()
            .asset_id,
        "icon.png"
    );

    // Check restored hierarchy
    assert_eq!(
        restored_world
            .get::<Parent>(restored_child)
            .unwrap()
            .entity(),
        restored_root
    );
    assert_eq!(
        restored_world
            .get::<Children>(restored_root)
            .unwrap()
            .as_slice(),
        &[restored_child]
    );
    assert_eq!(
        restored_world
            .get::<Parent>(restored_grandchild)
            .unwrap()
            .entity(),
        restored_child
    );
    assert_eq!(
        restored_world
            .get::<Children>(restored_child)
            .unwrap()
            .as_slice(),
        &[restored_grandchild]
    );
}

#[derive(Resource, Default)]
struct CounterRes {
    pre: u32,
    update: u32,
    post: u32,
}

fn pre_sys(mut c: ResMut<CounterRes>) {
    c.pre += 1;
}

fn update_sys(mut c: ResMut<CounterRes>) {
    c.update += 1;
}

fn post_sys(mut c: ResMut<CounterRes>) {
    c.post += 1;
}

#[test]
fn test_game_loop_and_time() {
    let mut world = World::new();
    world.insert_resource(CounterRes::default());

    let mut game_loop = GameLoop::new();
    game_loop.add_pre_update_system(pre_sys);
    game_loop.add_update_system(update_sys);
    game_loop.add_post_update_system(post_sys);

    // Frame 1: 1/120s (half a 60Hz tick) -> no ticks run, alpha = 0.5
    game_loop.step(&mut world, Duration::from_secs_f64(1.0 / 120.0));
    {
        let counter = world.resource::<CounterRes>();
        assert_eq!(counter.update, 0);
        let time = world.resource::<Time>();
        assert!((time.alpha - 0.5).abs() < 0.01);
        assert_eq!(time.tick_count, 0);
    }

    // Frame 2: Another 1/120s -> total 1/60s accumulated -> 1 tick runs, alpha ~ 0.0
    game_loop.step(&mut world, Duration::from_secs_f64(1.0 / 120.0));
    {
        let counter = world.resource::<CounterRes>();
        assert_eq!(counter.pre, 1);
        assert_eq!(counter.update, 1);
        assert_eq!(counter.post, 1);
        let time = world.resource::<Time>();
        assert_eq!(time.tick_count, 1);
        assert!(time.alpha < 0.01);
    }
}

#[derive(Clone, Debug, PartialEq)]
struct DamageEvent {
    target: String,
    amount: f32,
}

#[derive(Clone, Debug, PartialEq)]
struct ScoreEvent(u32);

#[test]
fn test_typed_event_bus() {
    let mut bus = EventBus::new();
    assert!(!bus.has_events::<DamageEvent>());

    bus.send(DamageEvent {
        target: "Hero".to_string(),
        amount: 25.0,
    });
    bus.send(DamageEvent {
        target: "Boss".to_string(),
        amount: 50.0,
    });
    bus.send(ScoreEvent(100));

    assert_eq!(bus.event_count::<DamageEvent>(), 2);
    assert_eq!(bus.event_count::<ScoreEvent>(), 1);

    let damage_events = bus.read::<DamageEvent>();
    assert_eq!(damage_events.len(), 2);
    assert_eq!(damage_events[0].amount, 25.0);

    let score_events = bus.read::<ScoreEvent>();
    assert_eq!(score_events, vec![ScoreEvent(100)]);

    bus.clear::<DamageEvent>();
    assert_eq!(bus.event_count::<DamageEvent>(), 0);
    assert_eq!(bus.event_count::<ScoreEvent>(), 1);

    bus.clear_all();
    assert_eq!(bus.event_count::<ScoreEvent>(), 0);
}

#[test]
fn test_protocol_scene_operations() {
    let mut world = World::new();

    // 1. Create root entity
    let mut comp_map = HashMap::new();
    comp_map.insert(
        "Transform2D".to_string(),
        "{\"x\": 5.0, \"y\": 10.0}".to_string(),
    );
    comp_map.insert(
        "Sprite".to_string(),
        "{\"asset_id\": \"player.png\", \"size\": [32.0, 32.0]}".to_string(),
    );

    let create_res = world
        .scene_create_entity(&SceneCreateEntityParams {
            parent_id: None,
            name: "Player".to_string(),
            components: Some(comp_map),
        })
        .expect("create entity should succeed");

    let player_id = create_res.entity_id;

    // 2. Create child entity
    let child_res = world
        .scene_create_entity(&SceneCreateEntityParams {
            parent_id: Some(player_id.clone()),
            name: "Weapon".to_string(),
            components: None,
        })
        .expect("create child entity should succeed");

    let weapon_id = child_res.entity_id;

    // 3. Query tree
    let tree = world
        .scene_get_tree(&SceneGetTreeParams {
            root_entity_id: None,
        })
        .expect("get_tree should succeed");

    assert_eq!(tree.root_entities.len(), 1);
    let root_node = &tree.root_entities[0];
    assert_eq!(root_node.id, player_id);
    assert_eq!(root_node.name, "Player");
    assert!(root_node.components.contains(&"Transform2D".to_string()));
    assert!(root_node.components.contains(&"Sprite".to_string()));
    assert_eq!(root_node.children.len(), 1);
    assert_eq!(root_node.children[0].id, weapon_id);

    // 4. Set component
    let mut update_data = HashMap::new();
    update_data.insert("x".to_string(), "99.0".to_string());
    update_data.insert("y".to_string(), "88.0".to_string());

    world
        .scene_set_component(&SceneSetComponentParams {
            entity_id: player_id.clone(),
            component_name: "Transform2D".to_string(),
            data: update_data,
        })
        .expect("set_component should succeed");

    let player_ent = find_entity_by_id(&world, &player_id).unwrap();
    let trans = world.get::<Transform2D>(player_ent).unwrap();
    assert_eq!(trans.position.x, 99.0);
    assert_eq!(trans.position.y, 88.0);

    // 5. Delete entity (cascades to child)
    let del_res = world
        .scene_delete_entity(&SceneDeleteEntityParams {
            entity_id: player_id.clone(),
        })
        .expect("delete_entity should succeed");

    assert!(del_res.success);
    assert!(find_entity_by_id(&world, &player_id).is_none());
    assert!(find_entity_by_id(&world, &weapon_id).is_none());

    // Tree should now be empty
    let empty_tree = world
        .scene_get_tree(&SceneGetTreeParams {
            root_entity_id: None,
        })
        .expect("get_tree on empty should succeed");
    assert_eq!(empty_tree.root_entities.len(), 0);
}

#[test]
fn test_templates_hello_scene_loading() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let scene_path = std::path::Path::new(manifest_dir).join("../../templates/hello/scene.json");

    let scene = Scene::load_from_file(&scene_path).expect("templates/hello/scene.json should load");
    assert_eq!(scene.name, "Hello Forge");
    assert_eq!(scene.entities.len(), 2);

    let mut world = World::new();
    let map = scene
        .spawn_into_world(&mut world)
        .expect("spawn_into_world should succeed for templates scene");

    assert_eq!(map.len(), 2);
    let camera_ent = map["camera_main"];
    let hero_ent = map["player_hero"];

    assert!(world.get::<Camera2D>(camera_ent).is_some());
    assert_eq!(world.get::<Name>(hero_ent).unwrap().as_str(), "Hero");
    assert_eq!(
        world.get::<Sprite>(hero_ent).unwrap().asset_id,
        "textures/hero.png"
    );
}
