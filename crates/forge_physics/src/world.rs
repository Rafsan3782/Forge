//! Physics world resource managing Rapier 2D simulation state and mapping.

use std::collections::{HashMap, HashSet};

use forge_core::ecs::{Entity, Resource};
use forge_core::events::EventBus;
use forge_core::Vec2;
use rapier2d::prelude as rapier;

use crate::events::{CollisionEvent, TriggerEvent};
use crate::query::{PhysicsHit, QueryFilter};

/// Physics simulation settings and physics world state.
#[derive(Resource)]
pub struct PhysicsWorld2D {
    /// World gravity vector in units/s^2 (default `[0.0, -980.0]` for pixel physics).
    pub gravity: Vec2,
    /// Integration parameters for Rapier.
    pub integration_parameters: rapier::IntegrationParameters,
    /// Rapier physics pipeline.
    pub physics_pipeline: rapier::PhysicsPipeline,
    /// Rapier island manager.
    pub island_manager: rapier::IslandManager,
    /// Rapier broad phase.
    pub broad_phase: rapier::DefaultBroadPhase,
    /// Rapier narrow phase.
    pub narrow_phase: rapier::NarrowPhase,
    /// Rapier rigid body set.
    pub bodies: rapier::RigidBodySet,
    /// Rapier collider set.
    pub colliders: rapier::ColliderSet,
    /// Rapier impulse joint set.
    pub impulse_joints: rapier::ImpulseJointSet,
    /// Rapier multibody joint set.
    pub multibody_joints: rapier::MultibodyJointSet,
    /// Rapier continuous collision detection solver.
    pub ccd_solver: rapier::CCDSolver,
    /// Rapier spatial query pipeline.
    pub query_pipeline: rapier::QueryPipeline,
    /// Mapping from ECS Entity to Rapier RigidBodyHandle.
    pub entity_to_body: HashMap<Entity, rapier::RigidBodyHandle>,
    /// Mapping from Rapier RigidBodyHandle to ECS Entity.
    pub body_to_entity: HashMap<rapier::RigidBodyHandle, Entity>,
    /// Mapping from ECS Entity to Rapier ColliderHandle.
    pub entity_to_collider: HashMap<Entity, rapier::ColliderHandle>,
    /// Mapping from Rapier ColliderHandle to ECS Entity.
    pub collider_to_entity: HashMap<rapier::ColliderHandle, Entity>,
    /// Set of active collisions from previous step: `(Entity, Entity)`.
    pub active_contacts: HashSet<(Entity, Entity)>,
    /// Set of active trigger overlaps from previous step: `(TriggerEntity, OtherEntity)`.
    pub active_triggers: HashSet<(Entity, Entity)>,
}

impl Default for PhysicsWorld2D {
    fn default() -> Self {
        Self::new(Vec2::new(0.0, -980.0))
    }
}

impl PhysicsWorld2D {
    /// Creates a new physics simulation world with the given gravity vector.
    pub fn new(gravity: Vec2) -> Self {
        let integration_parameters = rapier::IntegrationParameters {
            dt: 1.0 / 60.0,
            num_solver_iterations: 4.try_into().unwrap(),
            ..Default::default()
        };

        Self {
            gravity,
            integration_parameters,
            physics_pipeline: rapier::PhysicsPipeline::new(),
            island_manager: rapier::IslandManager::new(),
            broad_phase: rapier::DefaultBroadPhase::new(),
            narrow_phase: rapier::NarrowPhase::new(),
            bodies: rapier::RigidBodySet::new(),
            colliders: rapier::ColliderSet::new(),
            impulse_joints: rapier::ImpulseJointSet::new(),
            multibody_joints: rapier::MultibodyJointSet::new(),
            ccd_solver: rapier::CCDSolver::new(),
            query_pipeline: rapier::QueryPipeline::new(),
            entity_to_body: HashMap::new(),
            body_to_entity: HashMap::new(),
            entity_to_collider: HashMap::new(),
            collider_to_entity: HashMap::new(),
            active_contacts: HashSet::new(),
            active_triggers: HashSet::new(),
        }
    }

    /// Resets the physics simulation state entirely.
    pub fn clear(&mut self) {
        self.bodies = rapier::RigidBodySet::new();
        self.colliders = rapier::ColliderSet::new();
        self.impulse_joints = rapier::ImpulseJointSet::new();
        self.multibody_joints = rapier::MultibodyJointSet::new();
        self.ccd_solver = rapier::CCDSolver::new();
        self.island_manager = rapier::IslandManager::new();
        self.broad_phase = rapier::DefaultBroadPhase::new();
        self.narrow_phase = rapier::NarrowPhase::new();
        self.query_pipeline = rapier::QueryPipeline::new();
        self.entity_to_body.clear();
        self.body_to_entity.clear();
        self.entity_to_collider.clear();
        self.collider_to_entity.clear();
        self.active_contacts.clear();
        self.active_triggers.clear();
    }

    /// Removes a rigid body and its attached colliders from Rapier.
    pub fn remove_body(&mut self, handle: rapier::RigidBodyHandle) {
        self.bodies.remove(
            handle,
            &mut self.island_manager,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            true,
        );
    }

    /// Removes a collider from Rapier.
    pub fn remove_collider(&mut self, handle: rapier::ColliderHandle) {
        self.colliders
            .remove(handle, &mut self.island_manager, &mut self.bodies, true);
    }

    /// Inserts a collider with parent body.
    pub fn insert_collider_with_parent(
        &mut self,
        collider: rapier::Collider,
        parent: rapier::RigidBodyHandle,
    ) -> rapier::ColliderHandle {
        self.colliders
            .insert_with_parent(collider, parent, &mut self.bodies)
    }

    /// Casts a ray in world space and returns the first intersecting collider, if any.
    pub fn cast_ray(
        &self,
        origin: Vec2,
        direction: Vec2,
        max_distance: f32,
        filter: QueryFilter,
    ) -> Option<PhysicsHit> {
        let dir_len = direction.length();
        if dir_len <= 1e-6 {
            return None;
        }
        let norm_dir = direction / dir_len;
        let ray = rapier::Ray::new(
            rapier::point![origin.x, origin.y],
            rapier::vector![norm_dir.x, norm_dir.y],
        );

        let rapier_filter = rapier::QueryFilter {
            flags: if filter.query_sensors {
                rapier::QueryFilterFlags::empty()
            } else {
                rapier::QueryFilterFlags::EXCLUDE_SENSORS
            },
            groups: Some(rapier::InteractionGroups::new(
                rapier::Group::from_bits_truncate(filter.mask),
                rapier::Group::ALL,
            )),
            exclude_collider: filter
                .exclude_entity
                .and_then(|e| self.entity_to_collider.get(&e).copied()),
            exclude_rigid_body: filter
                .exclude_entity
                .and_then(|e| self.entity_to_body.get(&e).copied()),
            predicate: None,
        };

        if let Some((handle, toi)) = self.query_pipeline.cast_ray_and_get_normal(
            &self.bodies,
            &self.colliders,
            &ray,
            max_distance,
            true, // solid
            rapier_filter,
        ) {
            let entity = self.collider_to_entity.get(&handle).copied()?;
            let hit_point = origin + norm_dir * toi.time_of_impact;
            let normal = Vec2::new(toi.normal.x, toi.normal.y);

            Some(PhysicsHit {
                entity,
                point: hit_point,
                normal,
                distance: toi.time_of_impact,
                fraction: if max_distance > 0.0 {
                    toi.time_of_impact / max_distance
                } else {
                    0.0
                },
            })
        } else {
            None
        }
    }

    /// Finds all colliders intersecting a circle centered at `center` with `radius`.
    pub fn overlap_circle(&self, center: Vec2, radius: f32, filter: QueryFilter) -> Vec<Entity> {
        let shape = rapier::Ball::new(radius);
        let shape_pos = rapier::Isometry::translation(center.x, center.y);

        let rapier_filter = rapier::QueryFilter {
            flags: if filter.query_sensors {
                rapier::QueryFilterFlags::empty()
            } else {
                rapier::QueryFilterFlags::EXCLUDE_SENSORS
            },
            groups: Some(rapier::InteractionGroups::new(
                rapier::Group::from_bits_truncate(filter.mask),
                rapier::Group::ALL,
            )),
            exclude_collider: filter
                .exclude_entity
                .and_then(|e| self.entity_to_collider.get(&e).copied()),
            exclude_rigid_body: filter
                .exclude_entity
                .and_then(|e| self.entity_to_body.get(&e).copied()),
            predicate: None,
        };

        let mut intersecting_entities = Vec::new();
        self.query_pipeline.intersections_with_shape(
            &self.bodies,
            &self.colliders,
            &shape_pos,
            &shape,
            rapier_filter,
            |handle| {
                if let Some(&entity) = self.collider_to_entity.get(&handle) {
                    intersecting_entities.push(entity);
                }
                true
            },
        );

        intersecting_entities
    }

    /// Finds all colliders intersecting an axis-aligned box with given half-extents.
    pub fn overlap_box(
        &self,
        center: Vec2,
        half_extents: Vec2,
        filter: QueryFilter,
    ) -> Vec<Entity> {
        let shape = rapier::Cuboid::new(rapier::vector![half_extents.x, half_extents.y]);
        let shape_pos = rapier::Isometry::translation(center.x, center.y);

        let rapier_filter = rapier::QueryFilter {
            flags: if filter.query_sensors {
                rapier::QueryFilterFlags::empty()
            } else {
                rapier::QueryFilterFlags::EXCLUDE_SENSORS
            },
            groups: Some(rapier::InteractionGroups::new(
                rapier::Group::from_bits_truncate(filter.mask),
                rapier::Group::ALL,
            )),
            exclude_collider: filter
                .exclude_entity
                .and_then(|e| self.entity_to_collider.get(&e).copied()),
            exclude_rigid_body: filter
                .exclude_entity
                .and_then(|e| self.entity_to_body.get(&e).copied()),
            predicate: None,
        };

        let mut intersecting_entities = Vec::new();
        self.query_pipeline.intersections_with_shape(
            &self.bodies,
            &self.colliders,
            &shape_pos,
            &shape,
            rapier_filter,
            |handle| {
                if let Some(&entity) = self.collider_to_entity.get(&handle) {
                    intersecting_entities.push(entity);
                }
                true
            },
        );

        intersecting_entities
    }

    /// Steps the physics simulation and dispatches events to the `EventBus`.
    pub fn step_simulation(&mut self, event_bus: Option<&mut EventBus>) {
        let gravity_vec = rapier::vector![self.gravity.x, self.gravity.y];
        let physics_hooks = ();
        let event_handler = ();

        self.physics_pipeline.step(
            &gravity_vec,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &physics_hooks,
            &event_handler,
        );

        // Collect contacts and triggers from narrow phase
        let mut current_contacts = HashSet::new();
        let mut current_triggers = HashSet::new();

        for pair in self.narrow_phase.contact_pairs() {
            let col1 = pair.collider1;
            let col2 = pair.collider2;

            let (e1, e2) = match (
                self.collider_to_entity.get(&col1),
                self.collider_to_entity.get(&col2),
            ) {
                (Some(&a), Some(&b)) => (a, b),
                _ => continue,
            };

            let is_sensor1 = self
                .colliders
                .get(col1)
                .map(|c| c.is_sensor())
                .unwrap_or(false);
            let is_sensor2 = self
                .colliders
                .get(col2)
                .map(|c| c.is_sensor())
                .unwrap_or(false);

            if is_sensor1 || is_sensor2 {
                // Trigger event
                let (trigger_e, other_e) = if is_sensor1 { (e1, e2) } else { (e2, e1) };
                if pair.has_any_active_contact {
                    current_triggers.insert((trigger_e, other_e));
                }
            } else if pair.has_any_active_contact {
                // Ensure ordered key for undirected contact
                let key = if e1 < e2 { (e1, e2) } else { (e2, e1) };
                current_contacts.insert(key);
            }
        }

        if let Some(bus) = event_bus {
            // Check CollisionEvent: Started, Stay, Stopped
            for &(e1, e2) in &current_contacts {
                if !self.active_contacts.contains(&(e1, e2)) {
                    bus.send(CollisionEvent::Started {
                        entity1: e1,
                        entity2: e2,
                        normal: None,
                    });
                } else {
                    bus.send(CollisionEvent::Stay {
                        entity1: e1,
                        entity2: e2,
                    });
                }
            }
            for &(e1, e2) in &self.active_contacts {
                if !current_contacts.contains(&(e1, e2)) {
                    bus.send(CollisionEvent::Stopped {
                        entity1: e1,
                        entity2: e2,
                    });
                }
            }

            // Check TriggerEvent: Enter, Exit
            for &(te, oe) in &current_triggers {
                if !self.active_triggers.contains(&(te, oe)) {
                    bus.send(TriggerEvent::Enter {
                        trigger_entity: te,
                        other_entity: oe,
                    });
                }
            }
            for &(te, oe) in &self.active_triggers {
                if !current_triggers.contains(&(te, oe)) {
                    bus.send(TriggerEvent::Exit {
                        trigger_entity: te,
                        other_entity: oe,
                    });
                }
            }
        }

        self.active_contacts = current_contacts;
        self.active_triggers = current_triggers;
    }
}
