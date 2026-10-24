use crate::bindings::{bind_forge_api, ScriptCommand, ScriptExecutionContext};
use crate::error::ScriptExecutionError;
use crate::runtime::{LoadedScript, SandboxConfig, ScriptRuntime};
use forge_core::components::{Children, EntityId, Name, Parent, Script, Transform2D, Vec2};
use forge_core::ecs::{Commands, Entity, Query, Res, ResMut, Resource};
use forge_core::events::EventBus;
use forge_core::time::Time;
use forge_input::{ActionMap, InputState};
use forge_protocol::ScriptEntityTransform;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// Thread-safe wrapper ensuring Send + Sync for QuickJS runtime on single thread systems.
pub struct ThreadSafeRuntime(pub Arc<Mutex<ScriptRuntime>>);

// SAFETY: QuickJS runtime calls are guarded by a Mutex and only executed serially per frame.
unsafe impl Send for ThreadSafeRuntime {}
unsafe impl Sync for ThreadSafeRuntime {}

/// Resource storing the ScriptRuntime and compiled script cache.
#[derive(Resource)]
pub struct ScriptServer {
    pub runtime: ThreadSafeRuntime,
    pub exec_context: ScriptExecutionContext,
    pub loaded_scripts: HashMap<String, LoadedScript>,
    pub initialized_entities: HashSet<Entity>,
    pub script_mtimes: HashMap<String, std::time::SystemTime>,
    pub entity_script_states: HashMap<Entity, serde_json::Value>,
}

impl ScriptServer {
    /// Creates a new `ScriptServer`.
    pub fn new(config: SandboxConfig) -> Result<Self, String> {
        let exec_context = ScriptExecutionContext::new();
        let runtime = ScriptRuntime::new(config)?;
        bind_forge_api(runtime.context(), exec_context.clone())?;

        Ok(Self {
            runtime: ThreadSafeRuntime(Arc::new(Mutex::new(runtime))),
            exec_context,
            loaded_scripts: HashMap::new(),
            initialized_entities: HashSet::new(),
            script_mtimes: HashMap::new(),
            entity_script_states: HashMap::new(),
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

            if let Ok(meta) = std::fs::metadata(script_path) {
                if let Ok(mtime) = meta.modified() {
                    self.script_mtimes.insert(script_path.to_string(), mtime);
                }
            }

            let loaded = {
                let mut rt = self.runtime.0.lock().unwrap();
                rt.load_typescript_module(script_path, &source_ts)?
            };
            self.loaded_scripts.insert(script_path.to_string(), loaded);
        }

        Ok(self.loaded_scripts.get(script_path).unwrap())
    }

    /// Checks if any loaded scripts have been modified on disk, recompiling and reloading them.
    /// If serialize() and deserialize(state) are exported, state is preserved across the reload.
    /// A syntax error keeps the previous working version running and reports the error.
    pub fn check_and_reload_scripts(
        &mut self,
        entities: &[(Entity, String, String)], // (Entity, entity_id, script_path)
    ) -> Vec<ScriptExecutionError> {
        let mut errors = Vec::new();
        let paths: Vec<String> = self.loaded_scripts.keys().cloned().collect();

        for script_path in paths {
            let mtime = match std::fs::metadata(&script_path).and_then(|m| m.modified()) {
                Ok(t) => t,
                Err(_) => continue,
            };

            let last_mtime = self.script_mtimes.get(&script_path).copied();
            if last_mtime.is_some() && last_mtime == Some(mtime) {
                continue;
            }

            let source_ts = match std::fs::read_to_string(&script_path) {
                Ok(s) => s,
                Err(e) => {
                    errors.push(ScriptExecutionError {
                        entity_id: String::new(),
                        script_path: script_path.clone(),
                        message: format!(
                            "Failed to read modified script file {}: {}",
                            script_path, e
                        ),
                        stack: None,
                        file: Some(script_path.clone()),
                        line: None,
                        column: None,
                    });
                    continue;
                }
            };

            // Attempt to load & compile the new version
            let new_loaded = {
                let mut rt = self.runtime.0.lock().unwrap();
                rt.load_typescript_module(&script_path, &source_ts)
            };

            let new_loaded = match new_loaded {
                Ok(l) => l,
                Err(e) => {
                    // Syntax or compilation error: keep previous working version running and report error
                    let frame = self
                        .exec_context
                        .frame_count
                        .load(std::sync::atomic::Ordering::Relaxed);
                    self.exec_context.log_buffer.push(
                        "error",
                        &format!("Hot reload syntax error in {}: {}", script_path, e.message),
                        Some(script_path.clone()),
                        frame,
                    );
                    errors.push(e);
                    continue;
                }
            };

            // Script successfully compiled! Update last modified time
            self.script_mtimes.insert(script_path.clone(), mtime);

            let old_loaded = self.loaded_scripts.get(&script_path).cloned();

            // Find all entities running this script
            for (entity, id_str, path) in entities {
                if path != &script_path {
                    continue;
                }

                self.exec_context.set_current_entity(id_str);

                // If old module had serialize, serialize its state
                let state_val: Option<serde_json::Value> = if let Some(ref old) = old_loaded {
                    if old.has_serialize {
                        let mut rt = self.runtime.0.lock().unwrap();
                        let serialized = rt.context().with(|ctx| -> Option<serde_json::Value> {
                            let module = rquickjs::Module::declare(
                                ctx.clone(),
                                old.module_name.clone(),
                                old.transpiled_code.clone(),
                            )
                            .ok()?;
                            let (eval_module, promise) = module.eval().ok()?;
                            let _ = promise.finish::<()>();
                            let ns = eval_module.namespace().ok()?;
                            let func: rquickjs::Function = ns.get("serialize").ok()?;
                            let val: rquickjs::Value = func.call(()).ok()?;
                            let opt_str: Option<rquickjs::String> =
                                ctx.json_stringify(val).ok().flatten();
                            let s = opt_str?;
                            let json_str: String = s.to_string().ok()?;
                            serde_json::from_str(&json_str).ok()
                        });
                        serialized
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Store or transfer state
                if let Some(state) = state_val {
                    self.entity_script_states.insert(*entity, state);
                }
            }

            // Swap loaded script in cache
            self.loaded_scripts
                .insert(script_path.clone(), new_loaded.clone());

            // Post-reload: call deserialize(state) or onStart()
            for (entity, id_str, path) in entities {
                if path != &script_path {
                    continue;
                }

                self.exec_context.set_current_entity(id_str);

                if new_loaded.has_deserialize {
                    if let Some(state) = self.entity_script_states.get(entity) {
                        let mut rt = self.runtime.0.lock().unwrap();
                        let state_json = state.to_string();
                        let _ = rt
                            .context()
                            .with(|ctx| -> Result<(), ScriptExecutionError> {
                                let module = rquickjs::Module::declare(
                                    ctx.clone(),
                                    new_loaded.module_name.clone(),
                                    new_loaded.transpiled_code.clone(),
                                )
                                .map_err(|e| {
                                    rt.format_quickjs_error(
                                        ctx.clone(),
                                        &new_loaded.script_path,
                                        id_str,
                                        &new_loaded.source_mapper,
                                        e,
                                    )
                                })?;
                                let (eval_module, promise) = module.eval().map_err(|e| {
                                    rt.format_quickjs_error(
                                        ctx.clone(),
                                        &new_loaded.script_path,
                                        id_str,
                                        &new_loaded.source_mapper,
                                        e,
                                    )
                                })?;
                                let _ = promise.finish::<()>();
                                let ns = eval_module.namespace().map_err(|e| {
                                    rt.format_quickjs_error(
                                        ctx.clone(),
                                        &new_loaded.script_path,
                                        id_str,
                                        &new_loaded.source_mapper,
                                        e,
                                    )
                                })?;
                                let func: rquickjs::Function =
                                    ns.get("deserialize").map_err(|e| {
                                        rt.format_quickjs_error(
                                            ctx.clone(),
                                            &new_loaded.script_path,
                                            id_str,
                                            &new_loaded.source_mapper,
                                            e,
                                        )
                                    })?;
                                let parsed: rquickjs::Value =
                                    ctx.json_parse(state_json).map_err(|e| {
                                        rt.format_quickjs_error(
                                            ctx.clone(),
                                            &new_loaded.script_path,
                                            id_str,
                                            &new_loaded.source_mapper,
                                            e,
                                        )
                                    })?;
                                let _ = func.call::<_, ()>((parsed,));
                                Ok(())
                            });
                    }
                } else if new_loaded.has_on_start {
                    // Restart script instance by calling onStart
                    let mut rt = self.runtime.0.lock().unwrap();
                    let _ = rt.invoke_function::<(), ()>(id_str, &new_loaded, "onStart", ());
                }
            }

            let frame = self
                .exec_context
                .frame_count
                .load(std::sync::atomic::Ordering::Relaxed);
            self.exec_context.log_buffer.push(
                "info",
                &format!("Hot reloaded script: {}", script_path),
                Some(script_path.clone()),
                frame,
            );
        }

        errors
    }
}

/// ECS System: Updates input, time, and hierarchy snapshots in `ScriptExecutionContext`.
pub fn script_prepare_frame_system(
    time: Option<Res<Time>>,
    action_map: Option<Res<ActionMap>>,
    input_state: Option<Res<InputState>>,
    server: Option<ResMut<ScriptServer>>,
    entities_query: Query<(
        Entity,
        Option<&EntityId>,
        Option<&Name>,
        Option<&Transform2D>,
        Option<&Parent>,
        Option<&Children>,
    )>,
) {
    let Some(server) = server else { return };

    if let Some(t) = time {
        *server.exec_context.delta_time.lock().unwrap() = t.delta_seconds;
        *server.exec_context.elapsed_time.lock().unwrap() = t.elapsed_seconds as f32;
        server
            .exec_context
            .frame_count
            .store(t.tick_count, std::sync::atomic::Ordering::Relaxed);
    }

    if let (Some(map), Some(state)) = (action_map, input_state) {
        let mut pressed = Vec::new();
        let mut just_pressed = Vec::new();
        let mut just_released = Vec::new();
        let mut axes = HashMap::new();

        for action_name in map.actions.keys() {
            if map.action_pressed(action_name, &state) {
                pressed.push(action_name.clone());
            }
            if map.action_just_pressed(action_name, &state) {
                just_pressed.push(action_name.clone());
            }
            if map.action_just_released(action_name, &state) {
                just_released.push(action_name.clone());
            }
            axes.insert(action_name.clone(), map.action_axis(action_name, &state));
        }

        let mouse = state.mouse_position();
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
            .map(|ch| ch.as_slice().iter().map(|c| format!("{:?}", c)).collect())
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
    mut server: Option<ResMut<ScriptServer>>,
    mut event_bus: Option<ResMut<EventBus>>,
    mut query: Query<(Entity, Option<&EntityId>, &mut Script)>,
) {
    let Some(ref mut server) = server else { return };
    let mut to_run = Vec::new();
    let mut to_disable = Vec::new();

    for (entity, entity_id, script) in query.iter() {
        if !script.enabled || server.initialized_entities.contains(&entity) {
            continue;
        }

        server.initialized_entities.insert(entity);
        let id_str = entity_id
            .map(|id| id.0.clone())
            .unwrap_or_else(|| format!("{:?}", entity));

        match server.get_or_load_script(&script.path) {
            Ok(loaded) => {
                if loaded.has_on_start {
                    to_run.push((entity, id_str, loaded.clone()));
                }
            }
            Err(e) => {
                to_disable.push((entity, e));
            }
        }
    }

    for (entity, id_str, loaded) in to_run {
        server.exec_context.set_current_entity(&id_str);
        let mut rt = server.runtime.0.lock().unwrap();
        if let Err(e) = rt.invoke_function::<(), ()>(&id_str, &loaded, "onStart", ()) {
            to_disable.push((entity, e));
        }
    }

    for (entity, err) in to_disable {
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
    let Some(ref mut server) = server else { return };

    // Check for hot reloading modified scripts on disk
    let active_entities: Vec<(Entity, String, String)> = query
        .iter()
        .filter(|(_, _, s)| s.enabled)
        .map(|(e, id, s)| {
            let id_str = id
                .map(|i| i.0.clone())
                .unwrap_or_else(|| format!("{:?}", e));
            (e, id_str, s.path.clone())
        })
        .collect();

    let reload_errors = server.check_and_reload_scripts(&active_entities);
    for err in reload_errors {
        if let Some(ref mut bus) = event_bus {
            bus.send(err.to_event());
        }
    }

    let dt = time.map(|t| t.delta_seconds).unwrap_or(0.016);
    let mut to_run = Vec::new();
    let mut to_disable = Vec::new();

    for (entity, entity_id, script) in query.iter() {
        if !script.enabled {
            continue;
        }

        let id_str = entity_id
            .map(|id| id.0.clone())
            .unwrap_or_else(|| format!("{:?}", entity));

        match server.get_or_load_script(&script.path) {
            Ok(loaded) => {
                if loaded.has_on_update {
                    to_run.push((entity, id_str, loaded.clone()));
                }
            }
            Err(e) => {
                to_disable.push((entity, e));
            }
        }
    }

    for (entity, id_str, loaded) in to_run {
        server.exec_context.set_current_entity(&id_str);
        let mut rt = server.runtime.0.lock().unwrap();
        if let Err(e) = rt.invoke_function::<(f32,), ()>(&id_str, &loaded, "onUpdate", (dt,)) {
            to_disable.push((entity, e));
        }
    }

    for (entity, err) in to_disable {
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
    let Some(ref mut server) = server else { return };
    let dt = time.map(|t| t.fixed_delta_seconds).unwrap_or(0.02);
    let mut to_run = Vec::new();
    let mut to_disable = Vec::new();

    for (entity, entity_id, script) in query.iter() {
        if !script.enabled {
            continue;
        }

        let id_str = entity_id
            .map(|id| id.0.clone())
            .unwrap_or_else(|| format!("{:?}", entity));

        match server.get_or_load_script(&script.path) {
            Ok(loaded) => {
                if loaded.has_on_fixed_update {
                    to_run.push((entity, id_str, loaded.clone()));
                }
            }
            Err(e) => {
                to_disable.push((entity, e));
            }
        }
    }

    for (entity, id_str, loaded) in to_run {
        server.exec_context.set_current_entity(&id_str);
        let mut rt = server.runtime.0.lock().unwrap();
        if let Err(e) = rt.invoke_function::<(f32,), ()>(&id_str, &loaded, "onFixedUpdate", (dt,)) {
            to_disable.push((entity, e));
        }
    }

    for (entity, err) in to_disable {
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
    server: Option<ResMut<ScriptServer>>,
    mut query_transforms: Query<&mut Transform2D>,
    entities_query: Query<(Entity, &EntityId)>,
) {
    let Some(server) = server else { return };
    let queued = server.exec_context.take_commands();

    let id_map: HashMap<String, Entity> = entities_query
        .iter()
        .map(|(e, id)| (id.0.clone(), e))
        .collect();

    for cmd in queued {
        match cmd {
            ScriptCommand::SpawnEntity {
                name,
                components: _,
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
                entity_id: _,
                layer: _,
                coord: _,
                tile_id: _,
            } => {
                // Handled via tilemap cache and system
            }
            _ => {}
        }
    }
}
