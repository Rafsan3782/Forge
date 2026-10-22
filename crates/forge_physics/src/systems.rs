//! Synchronization systems between the ECS and Rapier physics simulation.

use forge_core::components::{BodyType2D, Collider2D, ColliderShape2D, RigidBody2D, Transform2D};
use forge_core::ecs::{Entity, World};
use forge_core::events::EventBus;
use forge_core::Vec2;
use rapier2d::control::{CharacterAutostep, CharacterLength, KinematicCharacterController};
use rapier2d::prelude as rapier;

use crate::controller::CharacterController2D;
use crate::world::PhysicsWorld2D;

/// Pre-step synchronization:
/// - Detects entities with `RigidBody2D` or `Collider2D` added or removed.
/// - Synchronizes ECS `Transform2D` into Rapier for kinematic bodies or repositioned static bodies.
/// - Moves character controllers via `KinematicCharacterController`.
pub fn physics_before_step_system(world: &mut World) {
    // 1. Gather all entities with RigidBody2D and/or Collider2D
    let mut ecs_bodies = Vec::new();
    let mut ecs_colliders = Vec::new();
    let mut ecs_controllers = Vec::new();

    for entity_ref in world.iter_entities() {
        let entity = entity_ref.id();
        let tf = entity_ref.get::<Transform2D>().copied().unwrap_or_default();
        let rb = entity_ref.get::<RigidBody2D>().cloned();
        let col = entity_ref.get::<Collider2D>().cloned();
        let ctl = entity_ref.get::<CharacterController2D>().cloned();

        if let Some(r) = rb {
            ecs_bodies.push((entity, tf, r));
        }
        if let Some(c) = col {
            ecs_colliders.push((entity, tf, c));
        }
        if let Some(c) = ctl {
            ecs_controllers.push((entity, tf, c));
        }
    }

    let mut physics = match world.get_resource_mut::<PhysicsWorld2D>() {
        Some(p) => p,
        None => return,
    };

    // 2. Remove despawned entities from Rapier
    let current_entities_with_physics: std::collections::HashSet<Entity> = ecs_bodies
        .iter()
        .map(|(e, _, _)| *e)
        .chain(ecs_colliders.iter().map(|(e, _, _)| *e))
        .collect();
    let to_remove_bodies: Vec<(Entity, rapier::RigidBodyHandle)> = physics
        .entity_to_body
        .iter()
        .filter(|(e, _)| !current_entities_with_physics.contains(e))
        .map(|(&e, &h)| (e, h))
        .collect();

    for (e, h) in to_remove_bodies {
        physics.remove_body(h);
        physics.entity_to_body.remove(&e);
        physics.body_to_entity.remove(&h);
    }

    let current_col_entities: std::collections::HashSet<Entity> =
        ecs_colliders.iter().map(|(e, _, _)| *e).collect();
    let to_remove_colliders: Vec<(Entity, rapier::ColliderHandle)> = physics
        .entity_to_collider
        .iter()
        .filter(|(e, _)| !current_col_entities.contains(e))
        .map(|(&e, &h)| (e, h))
        .collect();

    for (e, h) in to_remove_colliders {
        physics.remove_collider(h);
        physics.entity_to_collider.remove(&e);
        physics.collider_to_entity.remove(&h);
    }

    // 3. Sync or insert RigidBodies into Rapier
    for (entity, tf, rb) in ecs_bodies {
        let rapier_body_type = match rb.body_type {
            BodyType2D::Dynamic => rapier::RigidBodyType::Dynamic,
            BodyType2D::Kinematic => rapier::RigidBodyType::KinematicPositionBased,
            BodyType2D::Static => rapier::RigidBodyType::Fixed,
        };

        if let Some(&handle) = physics.entity_to_body.get(&entity) {
            if let Some(body) = physics.bodies.get_mut(handle) {
                if body.body_type() != rapier_body_type {
                    body.set_body_type(rapier_body_type, false);
                }
                if (body.gravity_scale() - rb.gravity_scale).abs() > 1e-5 {
                    body.set_gravity_scale(rb.gravity_scale, false);
                }
                if (body.linear_damping() - rb.linear_damping).abs() > 1e-5 {
                    body.set_linear_damping(rb.linear_damping);
                }
                if (body.angular_damping() - rb.angular_damping).abs() > 1e-5 {
                    body.set_angular_damping(rb.angular_damping);
                }

                let mut locked_axes = rapier::LockedAxes::empty();
                if rb.lock_rotation {
                    locked_axes |= rapier::LockedAxes::ROTATION_LOCKED;
                }
                if body.locked_axes() != locked_axes {
                    body.set_locked_axes(locked_axes, false);
                }
                match rb.body_type {
                    BodyType2D::Kinematic => {
                        let pos = rapier::Isometry::new(
                            rapier::vector![tf.position.x, tf.position.y],
                            tf.rotation,
                        );
                        body.set_next_kinematic_position(pos);
                    }
                    BodyType2D::Static => {
                        let pos = rapier::Isometry::new(
                            rapier::vector![tf.position.x, tf.position.y],
                            tf.rotation,
                        );
                        body.set_position(pos, true);
                    }
                    BodyType2D::Dynamic => {}
                }
            }
        } else {
            // Create new rigid body in Rapier
            let mut builder = match rb.body_type {
                BodyType2D::Dynamic => rapier::RigidBodyBuilder::dynamic(),
                BodyType2D::Kinematic => rapier::RigidBodyBuilder::kinematic_position_based(),
                BodyType2D::Static => rapier::RigidBodyBuilder::fixed(),
            };

            builder = builder
                .translation(rapier::vector![tf.position.x, tf.position.y])
                .rotation(tf.rotation)
                .gravity_scale(rb.gravity_scale)
                .linear_damping(rb.linear_damping)
                .angular_damping(rb.angular_damping)
                .can_sleep(rb.can_sleep);

            if rb.lock_rotation {
                builder = builder.lock_rotations();
            }

            let handle = physics.bodies.insert(builder.build());
            physics.entity_to_body.insert(entity, handle);
            physics.body_to_entity.insert(handle, entity);
        }
    }

    // 4. Sync or insert Colliders into Rapier
    for (entity, tf, col) in ecs_colliders {
        let parent_body_handle = physics.entity_to_body.get(&entity).copied();

        let shape = match col.shape {
            ColliderShape2D::Box { half_extents } => {
                let hx = half_extents.x * tf.scale.x.abs();
                let hy = half_extents.y * tf.scale.y.abs();
                rapier::SharedShape::cuboid(hx, hy)
            }
            ColliderShape2D::Circle { radius } => {
                let r = radius * tf.scale.x.abs().max(tf.scale.y.abs());
                rapier::SharedShape::ball(r)
            }
            ColliderShape2D::Capsule {
                radius,
                half_height,
            } => {
                let r = radius * tf.scale.x.abs();
                let hh = half_height * tf.scale.y.abs();
                rapier::SharedShape::capsule_y(hh, r)
            }
        };

        if let Some(&col_handle) = physics.entity_to_collider.get(&entity) {
            if let Some(rapier_col) = physics.colliders.get_mut(col_handle) {
                if rapier_col.is_sensor() != col.is_sensor {
                    rapier_col.set_sensor(col.is_sensor);
                }
                if (rapier_col.friction() - col.friction).abs() > 1e-5 {
                    rapier_col.set_friction(col.friction);
                }
                if (rapier_col.restitution() - col.restitution).abs() > 1e-5 {
                    rapier_col.set_restitution(col.restitution);
                }
                let target_groups = rapier::InteractionGroups::new(
                    rapier::Group::from_bits_truncate(col.collision_groups),
                    rapier::Group::from_bits_truncate(col.collision_mask),
                );
                if rapier_col.collision_groups() != target_groups {
                    rapier_col.set_collision_groups(target_groups);
                }

                if rapier_col.active_events() != rapier::ActiveEvents::COLLISION_EVENTS {
                    rapier_col.set_active_events(rapier::ActiveEvents::COLLISION_EVENTS);
                }
                rapier_col.set_active_collision_types(rapier::ActiveCollisionTypes::all());

                if parent_body_handle.is_none() {
                    // Update transform for standalone colliders
                    let pos = rapier::Isometry::new(
                        rapier::vector![tf.position.x + col.offset.x, tf.position.y + col.offset.y],
                        tf.rotation,
                    );
                    rapier_col.set_position(pos);
                } else {
                    let cur_pos = rapier_col
                        .position_wrt_parent()
                        .map(|p| p.translation.vector);
                    let target_pos = rapier::vector![col.offset.x, col.offset.y];
                    let needs_update = match cur_pos {
                        Some(p) => {
                            (p.x - target_pos.x).abs() > 1e-5 || (p.y - target_pos.y).abs() > 1e-5
                        }
                        None => true,
                    };
                    if needs_update {
                        rapier_col.set_position_wrt_parent(rapier::Isometry::translation(
                            target_pos.x,
                            target_pos.y,
                        ));
                    }
                }
            }
        } else {
            let builder = rapier::ColliderBuilder::new(shape)
                .sensor(col.is_sensor)
                .friction(col.friction)
                .restitution(col.restitution)
                .density(col.density)
                .active_events(rapier::ActiveEvents::COLLISION_EVENTS)
                .active_collision_types(rapier::ActiveCollisionTypes::all())
                .collision_groups(rapier::InteractionGroups::new(
                    rapier::Group::from_bits_truncate(col.collision_groups),
                    rapier::Group::from_bits_truncate(col.collision_mask),
                ))
                .translation(rapier::vector![col.offset.x, col.offset.y]);

            let col_handle = if let Some(body_handle) = parent_body_handle {
                physics.insert_collider_with_parent(builder.build(), body_handle)
            } else {
                // Standalone colliders without a RigidBody2D get an internal Fixed body in Rapier
                let fixed_body = rapier::RigidBodyBuilder::fixed()
                    .translation(rapier::vector![tf.position.x, tf.position.y])
                    .rotation(tf.rotation)
                    .build();
                let body_handle = physics.bodies.insert(fixed_body);
                physics.entity_to_body.insert(entity, body_handle);
                physics.body_to_entity.insert(body_handle, entity);
                physics.insert_collider_with_parent(builder.build(), body_handle)
            };

            physics.entity_to_collider.insert(entity, col_handle);
            physics.collider_to_entity.insert(col_handle, entity);
        }
    }

    // 5. Update Kinematic Character Controllers
    let mut updated_controllers = Vec::new();
    for (entity, mut tf, mut ctl) in ecs_controllers {
        if let Some(&col_handle) = physics.entity_to_collider.get(&entity) {
            let autostep = if ctl.autostep_max_height > 0.0 {
                Some(CharacterAutostep {
                    max_height: CharacterLength::Absolute(ctl.autostep_max_height),
                    min_width: CharacterLength::Absolute(ctl.autostep_min_width),
                    include_dynamic_bodies: true,
                })
            } else {
                None
            };

            let character_controller = KinematicCharacterController {
                up: nalgebra::Unit::new_normalize(rapier::vector![ctl.up.x, ctl.up.y]),
                offset: CharacterLength::Absolute(ctl.offset),
                slide: ctl.slide_enabled,
                autostep,
                max_slope_climb_angle: ctl.max_slope_angle,
                min_slope_slide_angle: ctl.min_slope_slide_angle,
                ..Default::default()
            };

            let desired_translation = rapier::vector![ctl.translation.x, ctl.translation.y];

            let rapier_filter = rapier::QueryFilter {
                flags: rapier::QueryFilterFlags::EXCLUDE_SENSORS,
                groups: None,
                exclude_collider: Some(col_handle),
                exclude_rigid_body: physics.entity_to_body.get(&entity).copied(),
                predicate: None,
            };

            if let Some(collider) = physics.colliders.get(col_handle) {
                let current_pos = collider.position();
                let effective_movement = character_controller.move_shape(
                    physics.integration_parameters.dt,
                    &physics.bodies,
                    &physics.colliders,
                    &physics.query_pipeline,
                    collider.shape(),
                    current_pos,
                    desired_translation,
                    rapier_filter,
                    |_| {},
                );

                ctl.is_grounded = effective_movement.grounded;
                ctl.is_sliding_down_slope = effective_movement.is_sliding_down_slope;

                tf.position.x += effective_movement.translation.x;
                tf.position.y += effective_movement.translation.y;

                if let Some(&body_handle) = physics.entity_to_body.get(&entity) {
                    if let Some(body) = physics.bodies.get_mut(body_handle) {
                        body.set_translation(rapier::vector![tf.position.x, tf.position.y], true);
                    }
                } else if let Some(collider) = physics.colliders.get_mut(col_handle) {
                    collider.set_translation(rapier::vector![tf.position.x, tf.position.y]);
                }

                updated_controllers.push((entity, tf, ctl));
            }
        }
    }

    // Apply updated controller transforms and states back to ECS
    for (entity, tf, ctl) in updated_controllers {
        if let Some(mut e_tf) = world.get_mut::<Transform2D>(entity) {
            *e_tf = tf;
        }
        if let Some(mut e_ctl) = world.get_mut::<CharacterController2D>(entity) {
            *e_ctl = ctl;
        }
    }
}

/// Simulation step system:
/// Advances the Rapier physics pipeline by one fixed timestep and emits collision/trigger events.
pub fn physics_step_system(world: &mut World) {
    if !world.contains_resource::<PhysicsWorld2D>() {
        return;
    }

    // Take EventBus temporarily or pass reference
    let mut event_bus = world.remove_resource::<EventBus>().unwrap_or_default();
    {
        if let Some(mut physics) = world.get_resource_mut::<PhysicsWorld2D>() {
            physics.step_simulation(Some(&mut event_bus));
        }
    }
    world.insert_resource(event_bus);
}

/// Post-step synchronization:
/// - Reads updated positions and rotations from Rapier dynamic bodies.
/// - Writes them back into ECS `Transform2D` and updates linear/angular velocities on `RigidBody2D`.
pub fn physics_after_step_system(world: &mut World) {
    let physics = match world.get_resource::<PhysicsWorld2D>() {
        Some(p) => p,
        None => return,
    };

    let mut sync_data = Vec::new();

    for (&entity, &body_handle) in &physics.entity_to_body {
        if let Some(body) = physics.bodies.get(body_handle) {
            if body.is_dynamic() {
                let trans = body.translation();
                let rot = body.rotation().angle();
                let linvel = body.linvel();
                let angvel = body.angvel();

                sync_data.push((
                    entity,
                    Vec2::new(trans.x, trans.y),
                    rot,
                    Vec2::new(linvel.x, linvel.y),
                    angvel,
                ));
            }
        }
    }

    for (entity, pos, rot, linvel, angvel) in sync_data {
        if let Some(mut tf) = world.get_mut::<Transform2D>(entity) {
            let prev = *tf;
            tf.position = pos;
            tf.rotation = rot;
            if let Some(mut prev_tf) = world.get_mut::<forge_core::PreviousTransform2D>(entity) {
                *prev_tf = forge_core::PreviousTransform2D::new(prev);
            } else {
                world
                    .entity_mut(entity)
                    .insert(forge_core::PreviousTransform2D::new(prev));
            }
        }
        if let Some(mut rb) = world.get_mut::<RigidBody2D>(entity) {
            rb.linear_velocity = linvel;
            rb.angular_velocity = angvel;
        }
    }
}
