//! Physics 2D integration tests for Rapier simulation, events, character controller, and determinism.

use forge_core::components::{BodyType2D, Collider2D, ColliderShape2D, RigidBody2D, Transform2D};
use forge_core::ecs::World;
use forge_core::events::EventBus;
use forge_core::Vec2;
use forge_physics::controller::CharacterController2D;
use forge_physics::events::TriggerEvent;
use forge_physics::query::QueryFilter;
use forge_physics::systems::{
    physics_after_step_system, physics_before_step_system, physics_step_system,
};
use forge_physics::world::PhysicsWorld2D;

fn setup_physics_world() -> World {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld2D::default());
    world.insert_resource(EventBus::new());
    world
}

fn step_simulation(world: &mut World) {
    physics_before_step_system(world);
    physics_step_system(world);
    physics_after_step_system(world);
}

#[test]
fn test_box_falls_and_rests_on_floor() {
    let mut world = setup_physics_world();

    // 1. Static floor at Y = 0 with height 20 (top surface at Y = 10)
    world.spawn((
        Transform2D {
            position: Vec2::new(0.0, 0.0),
            ..Default::default()
        },
        RigidBody2D {
            body_type: BodyType2D::Static,
            ..Default::default()
        },
        Collider2D {
            shape: ColliderShape2D::Box {
                half_extents: Vec2::new(200.0, 10.0),
            },
            friction: 0.5,
            restitution: 0.0,
            ..Default::default()
        },
    ));

    // 2. Dynamic falling box at Y = 100 with size 20x20 (half-extents 10, 10)
    let box_entity = world
        .spawn((
            Transform2D {
                position: Vec2::new(0.0, 100.0),
                ..Default::default()
            },
            RigidBody2D {
                body_type: BodyType2D::Dynamic,
                gravity_scale: 1.0,
                linear_damping: 0.1,
                ..Default::default()
            },
            Collider2D {
                shape: ColliderShape2D::Box {
                    half_extents: Vec2::new(10.0, 10.0),
                },
                friction: 0.5,
                restitution: 0.0,
                ..Default::default()
            },
        ))
        .id();

    // Step simulation for 60 frames (1 second at 60Hz)
    for _ in 0..60 {
        step_simulation(&mut world);
    }

    let final_tf = *world.get::<Transform2D>(box_entity).unwrap();
    let final_rb = world.get::<RigidBody2D>(box_entity).unwrap();

    // Box should have fallen and come to rest on top of the floor:
    // Floor top surface is at Y = 10, box half extent is 10, so box center should be ~ 20.0
    assert!(
        (final_tf.position.y - 20.0).abs() < 1.0,
        "Expected box Y position near 20.0, got {}",
        final_tf.position.y
    );
    assert!(
        final_rb.linear_velocity.y.abs() < 1.0,
        "Expected box Y velocity near 0, got {}",
        final_rb.linear_velocity.y
    );
}

#[test]
fn test_trigger_events_fire_once() {
    let mut world = setup_physics_world();

    // Sensor trigger zone centered at [0, 0] with half extents [50, 50]
    let trigger_entity = world
        .spawn((
            Transform2D {
                position: Vec2::new(0.0, 0.0),
                ..Default::default()
            },
            Collider2D {
                shape: ColliderShape2D::Box {
                    half_extents: Vec2::new(50.0, 50.0),
                },
                is_sensor: true,
                ..Default::default()
            },
        ))
        .id();

    // Dynamic object starting outside the trigger at [0, 100] with zero gravity
    let object_entity = world
        .spawn((
            Transform2D {
                position: Vec2::new(0.0, 100.0),
                ..Default::default()
            },
            RigidBody2D {
                body_type: BodyType2D::Kinematic,
                gravity_scale: 0.0,
                ..Default::default()
            },
            Collider2D {
                shape: ColliderShape2D::Box {
                    half_extents: Vec2::new(5.0, 5.0),
                },
                ..Default::default()
            },
        ))
        .id();

    // Frame 1: Object outside [0, 100]
    step_simulation(&mut world);
    {
        let bus = world.get_resource::<EventBus>().unwrap();
        let triggers = bus.read::<TriggerEvent>();
        assert_eq!(triggers.len(), 0, "No trigger event when outside");
    }

    // Frame 2: Move object inside trigger to [0, 0]
    {
        let mut tf = world.get_mut::<Transform2D>(object_entity).unwrap();
        tf.position = Vec2::new(0.0, 0.0);
    }
    step_simulation(&mut world);
    {
        let bus = world.get_resource::<EventBus>().unwrap();
        let triggers = bus.read::<TriggerEvent>();
        assert_eq!(
            triggers.len(),
            1,
            "Expected exactly 1 trigger event on enter"
        );
        match triggers[0] {
            TriggerEvent::Enter {
                trigger_entity: te,
                other_entity: oe,
            } => {
                assert_eq!(te, trigger_entity);
                assert_eq!(oe, object_entity);
            }
            _ => panic!("Expected TriggerEvent::Enter"),
        }
    }

    // Clear event bus for next step test
    world.get_resource_mut::<EventBus>().unwrap().clear_all();

    // Frame 3: Object stays inside trigger at [0, 0]
    step_simulation(&mut world);
    {
        let bus = world.get_resource::<EventBus>().unwrap();
        let triggers = bus.read::<TriggerEvent>();
        assert_eq!(
            triggers.len(),
            0,
            "No new trigger enter/exit event while staying inside"
        );
    }

    // Frame 4: Move object out of trigger to [0, 150]
    {
        let mut tf = world.get_mut::<Transform2D>(object_entity).unwrap();
        tf.position = Vec2::new(0.0, 150.0);
    }
    step_simulation(&mut world);
    {
        let bus = world.get_resource::<EventBus>().unwrap();
        let triggers = bus.read::<TriggerEvent>();
        assert_eq!(
            triggers.len(),
            1,
            "Expected exactly 1 trigger exit event on exit"
        );
        match triggers[0] {
            TriggerEvent::Exit {
                trigger_entity: te,
                other_entity: oe,
            } => {
                assert_eq!(te, trigger_entity);
                assert_eq!(oe, object_entity);
            }
            _ => panic!("Expected TriggerEvent::Exit"),
        }
    }
}

#[test]
fn test_raycast_hits_expected_collider() {
    let mut world = setup_physics_world();

    let target_entity = world
        .spawn((
            Transform2D {
                position: Vec2::new(100.0, 0.0),
                ..Default::default()
            },
            Collider2D {
                shape: ColliderShape2D::Box {
                    half_extents: Vec2::new(20.0, 20.0),
                },
                ..Default::default()
            },
        ))
        .id();

    step_simulation(&mut world);

    let physics = world.get_resource::<PhysicsWorld2D>().unwrap();
    let hit = physics.cast_ray(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        200.0,
        QueryFilter::default(),
    );

    assert!(hit.is_some(), "Raycast should have hit the box");
    let hit = hit.unwrap();
    assert_eq!(hit.entity, target_entity);
    // Box left edge is at X = 80 (100 - 20)
    assert!(
        (hit.point.x - 80.0).abs() < 0.1,
        "Expected hit point X near 80.0, got {}",
        hit.point.x
    );
    assert!(
        (hit.normal.x - (-1.0)).abs() < 0.1,
        "Expected hit normal facing left [-1, 0], got {:?}",
        hit.normal
    );
}

#[test]
fn test_character_controller_is_grounded_on_floor() {
    let mut world = setup_physics_world();

    // Static floor at Y = 0 (top surface at Y = 10)
    world.spawn((
        Transform2D {
            position: Vec2::new(0.0, 0.0),
            ..Default::default()
        },
        Collider2D {
            shape: ColliderShape2D::Box {
                half_extents: Vec2::new(200.0, 10.0),
            },
            ..Default::default()
        },
    ));

    // Character starting at Y = 30 (capsule radius 10, half-height 10 -> bottom is Y = 30 - 20 = 10)
    let char_entity = world
        .spawn((
            Transform2D {
                position: Vec2::new(0.0, 30.0),
                ..Default::default()
            },
            CharacterController2D {
                translation: Vec2::new(0.0, -5.0), // push downwards towards floor
                offset: 0.01,
                slide_enabled: true,
                autostep_max_height: 5.0,
                ..Default::default()
            },
            Collider2D {
                shape: ColliderShape2D::Capsule {
                    radius: 10.0,
                    half_height: 10.0,
                },
                ..Default::default()
            },
        ))
        .id();

    step_simulation(&mut world);

    let controller = world.get::<CharacterController2D>(char_entity).unwrap();
    assert!(
        controller.is_grounded,
        "Character controller should be grounded on floor"
    );
}

#[test]
fn test_determinism_across_runs() {
    fn run_simulation() -> Vec<(Vec2, f32)> {
        let mut world = setup_physics_world();

        // Floor
        world.spawn((
            Transform2D {
                position: Vec2::new(0.0, 0.0),
                ..Default::default()
            },
            RigidBody2D {
                body_type: BodyType2D::Static,
                ..Default::default()
            },
            Collider2D {
                shape: ColliderShape2D::Box {
                    half_extents: Vec2::new(500.0, 10.0),
                },
                friction: 0.3,
                restitution: 0.5,
                ..Default::default()
            },
        ));

        // Several falling dynamic bodies
        let mut entities = Vec::new();
        for i in 0..5 {
            let e = world
                .spawn((
                    Transform2D {
                        position: Vec2::new((i as f32) * 20.0 - 40.0, 100.0 + (i as f32) * 15.0),
                        rotation: (i as f32) * 0.1,
                        ..Default::default()
                    },
                    RigidBody2D {
                        body_type: BodyType2D::Dynamic,
                        gravity_scale: 1.0,
                        ..Default::default()
                    },
                    Collider2D {
                        shape: if i % 2 == 0 {
                            ColliderShape2D::Box {
                                half_extents: Vec2::new(10.0, 10.0),
                            }
                        } else {
                            ColliderShape2D::Circle { radius: 10.0 }
                        },
                        friction: 0.3,
                        restitution: 0.5,
                        ..Default::default()
                    },
                ))
                .id();
            entities.push(e);
        }

        // Simulate 100 ticks
        for _ in 0..100 {
            step_simulation(&mut world);
        }

        let mut results = Vec::new();
        for e in entities {
            let tf = *world.get::<Transform2D>(e).unwrap();
            results.push((tf.position, tf.rotation));
        }

        results
    }

    let run1 = run_simulation();
    let run2 = run_simulation();

    assert_eq!(
        run1.len(),
        run2.len(),
        "Both runs must have same entity count"
    );
    for i in 0..run1.len() {
        let (pos1, rot1) = run1[i];
        let (pos2, rot2) = run2[i];

        assert_eq!(
            pos1, pos2,
            "Entity {} position differed between runs: {:?} vs {:?}",
            i, pos1, pos2
        );
        assert_eq!(
            rot1, rot2,
            "Entity {} rotation differed between runs: {} vs {}",
            i, rot1, rot2
        );
    }
}
