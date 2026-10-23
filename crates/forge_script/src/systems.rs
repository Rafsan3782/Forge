//! Systems and state management for running scripts in the Forge ECS game loop.

use crate::bindings::{bind_forge_api, ScriptCommand, ScriptExecutionContext};
use crate::error::ScriptExecutionError;
use crate::runtime::{LoadedScript, SandboxConfig, ScriptRuntime};
use forge_core::components::{Children, EntityId, Name, Parent, Script, Transform2D, Vec2};
use forge_core::ecs::{Commands, Entity, Query, Res, ResMut, Resource, World};
use forge_core::events::EventBus;
use forge_core::time::Time;
use forge_input::InputMap;
use forge_physics::events::{CollisionEvent, TriggerEvent};
use forge_protocol::ScriptEntityTransform;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Resource storing the ScriptRuntime and compiled script cache.
#[derive(Resource)]
pub struct ScriptServer {
    pub runtime: ScriptRuntime,
    pub exec_context: ScriptExecutionContext,
    pub loaded_scripts: HashMap<String, LoadedScript>,
    pub initialized_entities: HashSet<Entity>,
}

impl ScriptServer {
    /// Creates a new `ScriptServer`.
    pub fn new(config: SandboxConfig) -> Result<Self, String> {
        let exec_context = ScriptExecutionContext::new();
        let runtime = ScriptRuntime::new(config)?;
        bind_forge_api(runtime.context(), exec_context.clone())?;

        Ok(Self {
            runtime,
            exec_context,
            loaded_scripts: HashMap::new(),
            initialized_entities: HashSet::new(),
        })
    }

    /// Loads or retrieves a loaded script by file path.
    pub fn get_or_load_script(
        &mut self,
        script_path: &str,
    ) -> Result<&LoadedScript, ScriptExecutionError> {
        if !self.loaded_scripts.contains_key(script_path) {
            let source_ts =
                std::fs::read_to_string(script_path).map_err(|e| ScriptExecutionError {
                    entity_id: String::new(),
                    script_path: script_path.to_string(),
                    message: format!("Failed to read script file {}: {}", script_path, e),
                    stack: None,
                    file: Some(script_path.to_string()),
                    line: None,
                    column: None,
                })?;

            let loaded = self
                .runtime
                .load_typescript_module(script_path, &source_ts)?;
            self.loaded_scripts.insert(script_path.to_string(), loaded);
        }

        Ok(self.loaded_scripts.get(script_path).unwrap())
    }
}

/// ECS System: Updates input, time, and hierarchy snapshots in `ScriptExecutionContext`.
pub fn script_prepare_frame_system(
    time: Option<Res<Time>>,
    input_map: Option<Res<InputMap>>,
    mut server: Option<ResMut<ScriptServer>>,
    entities_query: Query<(
        Entity,
        Option<&EntityId>,
        Option<&Name>,
        Option<&Transform2D>,
        Option<&Parent>,
        Option<&Children>,
    )>,
) {
    let Some(mut server) = server else { return };

    if let Some(t) = time {
        *server.exec_context.delta_time.lock().unwrap() = t.delta_seconds();
        *server.exec_context.elapsed_time.lock().unwrap() = t.elapsed_seconds() as f32;
        server
            .exec_context
            .frame_count
            .store(t.frame_count(), std::sync::atomic::Ordering::Relaxed);
    }

    if let Some(input) = input_map {
        let mut pressed = Vec::new();
        let mut just_pressed = Vec::new();
        let mut just_released = Vec::new();
        let mut axes = HashMap::new();

        for action in input.actions() {
            if input.is_action_pressed(action) {
                pressed.push(action.to_string());
            }
            if input.is_action_just_pressed(action) {
                just_pressed.push(action.to_string());
            }
            if input.is_action_just_released(action) {
                just_released.push(action.to_string());
            }
        }

        for axis in input.axes() {
            axes.insert(axis.to_string(), input.get_axis(axis));
        }

        let mouse = input.mouse_position();
        *server.exec_context.mouse_pos.lock().unwrap() = [mouse.x, mouse.y];
        *server.exec_context.pressed_actions.lock().unwrap() = pressed;
        *server.exec_context.just_pressed_actions.lock().unwrap() = just_pressed;
        *server.exec_context.just_released_actions.lock().unwrap() = just_released;
        *server.exec_context.axes.lock().unwrap() = axes;
    }

    // Refresh hierarchy & transform cache
    let mut hier = HashMap::new();
    let mut trans = HashMap::new();

    for (_e, entity_id, name, transform, parent, children) in entities_query.iter() {
        let id_str = entity_id.map(|id| id.0.clone()).unwrap_or_default();
        let name_str = name.map(|n| n.as_str().to_string()).unwrap_or_default();
        let parent_id = parent.map(|p| format!("{:?}", p.entity()));
        let children_ids = children
            .map(|ch| ch.iter().map(|c| format!("{:?}", c)).collect())
            .unwrap_or_default();

        hier.insert(id_str.clone(), (name_str, parent_id, children_ids));

        if let Some(t) = transform {
            trans.insert(
                id_str,
                ScriptEntityTransform {
                    position: [t.position.x, t.position.y],
                    rotation: t.rotation,
                    scale: [t.scale.x, t.scale.y],
                },
            );
        }
    }

    *server.exec_context.entity_hierarchy.lock().unwrap() = hier;
    *server.exec_context.transform_cache.lock().unwrap() = trans;
}

/// ECS System: Dispatches `onStart` for newly registered scripts.
pub fn script_start_system(
    mut commands: Commands,
    mut server: Option<ResMut<ScriptServer>>,
    mut event_bus: Option<ResMut<EventBus>>,
    mut query: Query<(Entity, Option<&EntityId>, &mut Script)>,
) {
    let Some(mut server) = server else { return };
    let mut errors = Vec::new();

    for (entity, entity_id, mut script) in query.iter_mut() {
        if !script.enabled || server.initialized_entities.contains(&entity) {
            continue;
        }

        server.initialized_entities.insert(entity);
        let id_str = entity_id
            .map(|id| id.0.clone())
            .unwrap_or_else(|| format!("{:?}", entity));
        server.exec_context.set_current_entity(&id_str);

        match server.get_or_load_script(&script.path) {
            Ok(loaded) => {
                if loaded.has_on_start {
                    if let Err(e) =
                        server
                            .runtime
                            .invoke_function::<(), ()>(&id_str, loaded, "onStart", ())
                    {
                        errors.push((entity, e));
                    }
                }
            }
            Err(e) => {
                errors.push((entity, e));
            }
        }
    }

    for (entity, err) in errors {
        tracing::error!("[script error] {}", err.message);
        if let Some(ref mut bus) = event_bus {
            bus.send(err.to_event());
        }
        if let Ok((_, _, mut s)) = query.get_mut(entity) {
            s.disable();
        }
    }
}

/// ECS System: Dispatches `onUpdate(dt)` each frame.
pub fn script_update_system(
    time: Option<Res<Time>>,
    mut server: Option<ResMut<ScriptServer>>,
    mut event_bus: Option<ResMut<EventBus>>,
    mut query: Query<(Entity, Option<&EntityId>, &mut Script)>,
) {
    let Some(mut server) = server else { return };
    let dt = time.map(|t| t.delta_seconds()).unwrap_or(0.016);
    let mut errors = Vec::new();

    for (entity, entity_id, mut script) in query.iter_mut() {
        if !script.enabled {
            continue;
        }

        let id_str = entity_id
            .map(|id| id.0.clone())
            .unwrap_or_else(|| format!("{:?}", entity));
        server.exec_context.set_current_entity(&id_str);

        match server.get_or_load_script(&script.path) {
            Ok(loaded) => {
                if loaded.has_on_update {
                    if let Err(e) = server.runtime.invoke_function::<(f32,), ()>(
                        &id_str,
                        loaded,
                        "onUpdate",
                        (dt,),
                    ) {
                        errors.push((entity, e));
                    }
                }
            }
            Err(e) => {
                errors.push((entity, e));
            }
        }
    }

    for (entity, err) in errors {
        tracing::error!("[script error] {}", err.message);
        if let Some(ref mut bus) = event_bus {
            bus.send(err.to_event());
        }
        if let Ok((_, _, mut s)) = query.get_mut(entity) {
            s.disable();
        }
    }
}

/// ECS System: Dispatches `onFixedUpdate(dt)`.
pub fn script_fixed_update_system(
    time: Option<Res<Time>>,
    mut server: Option<ResMut<ScriptServer>>,
    mut event_bus: Option<ResMut<EventBus>>,
    mut query: Query<(Entity, Option<&EntityId>, &mut Script)>,
) {
    let Some(mut server) = server else { return };
    let dt = time.map(|t| t.fixed_delta_seconds()).unwrap_or(0.02);
    let mut errors = Vec::new();

    for (entity, entity_id, mut script) in query.iter_mut() {
        if !script.enabled {
            continue;
        }

        let id_str = entity_id
            .map(|id| id.0.clone())
            .unwrap_or_else(|| format!("{:?}", entity));
        server.exec_context.set_current_entity(&id_str);

        match server.get_or_load_script(&script.path) {
            Ok(loaded) => {
                if loaded.has_on_fixed_update {
                    if let Err(e) = server.runtime.invoke_function::<(f32,), ()>(
                        &id_str,
                        loaded,
                        "onFixedUpdate",
                        (dt,),
                    ) {
                        errors.push((entity, e));
                    }
                }
            }
            Err(e) => {
                errors.push((entity, e));
            }
        }
    }

    for (entity, err) in errors {
        tracing::error!("[script error] {}", err.message);
        if let Some(ref mut bus) = event_bus {
            bus.send(err.to_event());
        }
        if let Ok((_, _, mut s)) = query.get_mut(entity) {
            s.disable();
        }
    }
}

/// ECS System: Applies script commands (spawn, transform updates, audio, tilemaps) back to World.
pub fn script_apply_commands_system(
    mut commands: Commands,
    mut server: Option<ResMut<ScriptServer>>,
    mut query_transforms: Query<&mut Transform2D>,
    entities_query: Query<(Entity, &EntityId)>,
) {
    let Some(mut server) = server else { return };
    let queued = server.exec_context.take_commands();

    let id_map: HashMap<String, Entity> = entities_query
        .iter()
        .map(|(e, id)| (id.0.clone(), e))
        .collect();

    for cmd in queued {
        match cmd {
            ScriptCommand::SpawnEntity {
                name,
                components,
                parent,
                result_id,
            } => {
                let mut e_cmd = commands.spawn((
                    EntityId::new(result_id.clone()),
                    Name::new(name),
                    Transform2D::default(),
                ));

                if let Some(parent_id) = parent {
                    if let Some(&p_entity) = id_map.get(&parent_id) {
                        e_cmd.insert(Parent::new(p_entity));
                    }
                }
            }
            ScriptCommand::DestroyEntity { entity_id } => {
                if let Some(&entity) = id_map.get(&entity_id) {
                    commands.entity(entity).despawn();
                }
            }
            ScriptCommand::SetTransform {
                entity_id,
                transform,
            } => {
                if let Some(&entity) = id_map.get(&entity_id) {
                    if let Ok(mut t) = query_transforms.get_mut(entity) {
                        t.position = Vec2::new(transform.position[0], transform.position[1]);
                        t.rotation = transform.rotation;
                        t.scale = Vec2::new(transform.scale[0], transform.scale[1]);
                    }
                }
            }
            ScriptCommand::SetTile {
                entity_id,
                layer,
                coord,
                tile_id,
            } => {
                // Handled via tilemap cache and system
            }
            _ => {}
        }
    }
}
