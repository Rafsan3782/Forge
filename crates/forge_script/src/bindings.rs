use forge_protocol::ScriptEntityTransform;
use rquickjs::{Context, Function, Object, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Commands queued by script executions to be applied safely to the ECS `World`.
#[derive(Debug, Clone)]
pub enum ScriptCommand {
    SpawnEntity {
        name: String,
        components: Option<serde_json::Value>,
        parent: Option<String>,
        result_id: String,
    },
    DestroyEntity {
        entity_id: String,
    },
    LoadScene {
        scene_path: String,
    },
    SetTransform {
        entity_id: String,
        transform: ScriptEntityTransform,
    },
    SetComponent {
        entity_id: String,
        component_name: String,
        data: serde_json::Value,
    },
    PlayAudio {
        asset_id: String,
        bus: String,
        volume: f32,
        pitch: f32,
        r#loop: bool,
    },
    StopAudio {
        playback_id: Option<String>,
        bus: Option<String>,
    },
    SetAudioBusVolume {
        bus: String,
        volume: f32,
    },
    PlayAnimation {
        entity_id: Option<String>,
        clip_name: String,
    },
    SetTile {
        entity_id: String,
        layer: String,
        coord: [i32; 2],
        tile_id: i32,
    },
}

/// Shared execution context providing world queries and command queueing.
#[derive(Clone)]
pub struct ScriptExecutionContext {
    pub current_entity_id: Arc<Mutex<String>>,
    pub command_queue: Arc<Mutex<Vec<ScriptCommand>>>,
    // Snapshotted frame state for read operations
    pub delta_time: Arc<Mutex<f32>>,
    pub elapsed_time: Arc<Mutex<f32>>,
    pub frame_count: Arc<AtomicU64>,
    pub mouse_pos: Arc<Mutex<[f32; 2]>>,
    // Input state snapshot
    pub pressed_actions: Arc<Mutex<Vec<String>>>,
    pub just_pressed_actions: Arc<Mutex<Vec<String>>>,
    pub just_released_actions: Arc<Mutex<Vec<String>>>,
    pub axes: Arc<Mutex<std::collections::HashMap<String, f32>>>,
    // Tilemap query cache: (entity_id, layer, x, y) -> tile_id
    pub tile_cache: Arc<Mutex<std::collections::HashMap<(String, String, i32, i32), i32>>>,
    // Entity transform snapshot cache
    pub transform_cache: Arc<Mutex<std::collections::HashMap<String, ScriptEntityTransform>>>,
    // Entity hierarchy snapshot: entity_id -> (name, Option<parent_id>, children_ids)
    pub entity_hierarchy:
        Arc<Mutex<std::collections::HashMap<String, (String, Option<String>, Vec<String>)>>>,
}

impl ScriptExecutionContext {
    pub fn new() -> Self {
        Self {
            current_entity_id: Arc::new(Mutex::new(String::new())),
            command_queue: Arc::new(Mutex::new(Vec::new())),
            delta_time: Arc::new(Mutex::new(0.016)),
            elapsed_time: Arc::new(Mutex::new(0.0)),
            frame_count: Arc::new(AtomicU64::new(0)),
            mouse_pos: Arc::new(Mutex::new([0.0, 0.0])),
            pressed_actions: Arc::new(Mutex::new(Vec::new())),
            just_pressed_actions: Arc::new(Mutex::new(Vec::new())),
            just_released_actions: Arc::new(Mutex::new(Vec::new())),
            axes: Arc::new(Mutex::new(std::collections::HashMap::new())),
            tile_cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
            transform_cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
            entity_hierarchy: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn set_current_entity(&self, entity_id: &str) {
        let mut id = self.current_entity_id.lock().unwrap();
        *id = entity_id.to_string();
    }

    pub fn take_commands(&self) -> Vec<ScriptCommand> {
        let mut q = self.command_queue.lock().unwrap();
        std::mem::take(&mut *q)
    }
}

impl Default for ScriptExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Binds the global `forge` object and all sub-namespaces into QuickJS context.
pub fn bind_forge_api(context: &Context, exec_ctx: ScriptExecutionContext) -> Result<(), String> {
    context
        .with(|ctx| {
            let global = ctx.globals();
            let forge = Object::new(ctx)?;

            // 1. forge.entity
            let entity_obj = Object::new(ctx)?;
            {
                let ec = exec_ctx.clone();
                entity_obj.set(
                    "getId",
                    Function::new(ctx, move || -> String {
                        ec.current_entity_id.lock().unwrap().clone()
                    })?,
                )?;
            }
            {
                let ec = exec_ctx.clone();
                entity_obj.set(
                    "getName",
                    Function::new(ctx, move || -> String {
                        let id = ec.current_entity_id.lock().unwrap().clone();
                        let hier = ec.entity_hierarchy.lock().unwrap();
                        hier.get(&id)
                            .map(|(name, _, _)| name.clone())
                            .unwrap_or_default()
                    })?,
                )?;
            }
            {
                let ec = exec_ctx.clone();
                entity_obj.set(
                    "getParent",
                    Function::new(ctx, move || -> Option<String> {
                        let id = ec.current_entity_id.lock().unwrap().clone();
                        let hier = ec.entity_hierarchy.lock().unwrap();
                        hier.get(&id).and_then(|(_, parent, _)| parent.clone())
                    })?,
                )?;
            }
            {
                let ec = exec_ctx.clone();
                entity_obj.set(
                    "getChildren",
                    Function::new(ctx, move || -> Vec<String> {
                        let id = ec.current_entity_id.lock().unwrap().clone();
                        let hier = ec.entity_hierarchy.lock().unwrap();
                        hier.get(&id)
                            .map(|(_, _, ch)| ch.clone())
                            .unwrap_or_default()
                    })?,
                )?;
            }
            {
                let ec = exec_ctx.clone();
                entity_obj.set(
                    "getTransform",
                    Function::new(ctx, move || -> Object<'_> {
                        let id = ec.current_entity_id.lock().unwrap().clone();
                        let cache = ec.transform_cache.lock().unwrap();
                        let trans = cache.get(&id).cloned().unwrap_or(ScriptEntityTransform {
                            position: [0.0, 0.0],
                            rotation: 0.0,
                            scale: [1.0, 1.0],
                        });

                        let obj = Object::new(ctx).unwrap();
                        let pos = Object::new(ctx).unwrap();
                        let _ = pos.set(0, trans.position[0]);
                        let _ = pos.set(1, trans.position[1]);
                        let scale = Object::new(ctx).unwrap();
                        let _ = scale.set(0, trans.scale[0]);
                        let _ = scale.set(1, trans.scale[1]);

                        let _ = obj.set("position", pos);
                        let _ = obj.set("rotation", trans.rotation);
                        let _ = obj.set("scale", scale);
                        obj
                    })?,
                )?;
            }
            {
                let ec = exec_ctx.clone();
                entity_obj.set(
                    "setTransform",
                    Function::new(ctx, move |trans: Object<'_>| {
                        let id = ec.current_entity_id.lock().unwrap().clone();
                        let mut current = {
                            let cache = ec.transform_cache.lock().unwrap();
                            cache.get(&id).cloned().unwrap_or(ScriptEntityTransform {
                                position: [0.0, 0.0],
                                rotation: 0.0,
                                scale: [1.0, 1.0],
                            })
                        };

                        if let Ok(pos_val) = trans.get::<_, Object>("position") {
                            if let (Ok(x), Ok(y)) =
                                (pos_val.get::<_, f32>(0), pos_val.get::<_, f32>(1))
                            {
                                current.position = [x, y];
                            }
                        }
                        if let Ok(rot) = trans.get::<_, f32>("rotation") {
                            current.rotation = rot;
                        }
                        if let Ok(scale_val) = trans.get::<_, Object>("scale") {
                            if let (Ok(sx), Ok(sy)) =
                                (scale_val.get::<_, f32>(0), scale_val.get::<_, f32>(1))
                            {
                                current.scale = [sx, sy];
                            }
                        }

                        ec.transform_cache
                            .lock()
                            .unwrap()
                            .insert(id.clone(), current.clone());
                        ec.command_queue
                            .lock()
                            .unwrap()
                            .push(ScriptCommand::SetTransform {
                                entity_id: id,
                                transform: current,
                            });
                    })?,
                )?;
            }

            forge.set("entity", entity_obj)?;

            // 2. forge.spawn & forge.destroy
            {
                let ec = exec_ctx.clone();
                forge.set(
                    "spawn",
                    Function::new(
                        ctx,
                        move |name: String,
                              comp: Option<Value<'_>>,
                              parent: Option<String>|
                              -> String {
                            let new_id = format!("entity_{}", rand_id());
                            let components = comp.and_then(|c| {
                                if let Ok(json_str) = c.ctx().json_stringify(c) {
                                    if let Some(s) = json_str {
                                        return serde_json::from_str(
                                            &s.to_string().unwrap_or_default(),
                                        )
                                        .ok();
                                    }
                                }
                                None
                            });

                            ec.command_queue
                                .lock()
                                .unwrap()
                                .push(ScriptCommand::SpawnEntity {
                                    name,
                                    components,
                                    parent,
                                    result_id: new_id.clone(),
                                });

                            new_id
                        },
                    )?,
                )?;
            }
            {
                let ec = exec_ctx.clone();
                forge.set(
                    "destroy",
                    Function::new(ctx, move |entity_id: String| {
                        ec.command_queue
                            .lock()
                            .unwrap()
                            .push(ScriptCommand::DestroyEntity { entity_id });
                    })?,
                )?;
            }

            // 3. forge.scene
            let scene_obj = Object::new(ctx)?;
            {
                let ec = exec_ctx.clone();
                scene_obj.set(
                    "load",
                    Function::new(ctx, move |scene_path: String| {
                        ec.command_queue
                            .lock()
                            .unwrap()
                            .push(ScriptCommand::LoadScene { scene_path });
                    })?,
                )?;
            }
            forge.set("scene", scene_obj)?;

            // 4. forge.input
            let input_obj = Object::new(ctx)?;
            {
                let ec = exec_ctx.clone();
                input_obj.set(
                    "isActionPressed",
                    Function::new(ctx, move |action: String| -> bool {
                        let pressed = ec.pressed_actions.lock().unwrap();
                        pressed.contains(&action)
                    })?,
                )?;
            }
            {
                let ec = exec_ctx.clone();
                input_obj.set(
                    "isActionJustPressed",
                    Function::new(ctx, move |action: String| -> bool {
                        let pressed = ec.just_pressed_actions.lock().unwrap();
                        pressed.contains(&action)
                    })?,
                )?;
            }
            {
                let ec = exec_ctx.clone();
                input_obj.set(
                    "isActionJustReleased",
                    Function::new(ctx, move |action: String| -> bool {
                        let released = ec.just_released_actions.lock().unwrap();
                        released.contains(&action)
                    })?,
                )?;
            }
            {
                let ec = exec_ctx.clone();
                input_obj.set(
                    "getAxis",
                    Function::new(ctx, move |axis: String| -> f32 {
                        let axes = ec.axes.lock().unwrap();
                        axes.get(&axis).copied().unwrap_or(0.0)
                    })?,
                )?;
            }
            {
                let ec = exec_ctx.clone();
                input_obj.set(
                    "getMousePosition",
                    Function::new(ctx, move || -> Vec<f32> {
                        let m = ec.mouse_pos.lock().unwrap();
                        vec![m[0], m[1]]
                    })?,
                )?;
            }
            forge.set("input", input_obj)?;

            // 5. forge.time
            let time_obj = Object::new(ctx)?;
            {
                let ec = exec_ctx.clone();
                time_obj.set(
                    "deltaTime",
                    Function::new(ctx, move || -> f32 { *ec.delta_time.lock().unwrap() })?,
                )?;
            }
            {
                let ec = exec_ctx.clone();
                time_obj.set(
                    "elapsedTime",
                    Function::new(ctx, move || -> f32 { *ec.elapsed_time.lock().unwrap() })?,
                )?;
            }
            {
                let ec = exec_ctx.clone();
                time_obj.set(
                    "frameCount",
                    Function::new(ctx, move || -> u64 {
                        ec.frame_count.load(Ordering::Relaxed)
                    })?,
                )?;
            }
            forge.set("time", time_obj)?;

            // 6. forge.physics
            let physics_obj = Object::new(ctx)?;
            {
                physics_obj.set(
                    "raycast",
                    Function::new(
                        ctx,
                        |_origin: Value<'_>, _dir: Value<'_>, _dist: f32| -> Option<Object<'_>> {
                            None
                        },
                    )?,
                )?;
                physics_obj.set(
                    "overlapCircle",
                    Function::new(ctx, |_center: Value<'_>, _radius: f32| -> Vec<String> {
                        Vec::new()
                    })?,
                )?;
            }
            forge.set("physics", physics_obj)?;

            // 7. forge.audio
            let audio_obj = Object::new(ctx)?;
            {
                let ec = exec_ctx.clone();
                audio_obj.set(
                    "play",
                    Function::new(
                        ctx,
                        move |asset_id: String, options: Option<Object<'_>>| -> String {
                            let mut bus = "sfx".to_string();
                            let mut vol = 1.0;
                            let mut pitch = 1.0;
                            let mut r#loop = false;

                            if let Some(opts) = options {
                                if let Ok(b) = opts.get::<_, String>("bus") {
                                    bus = b;
                                }
                                if let Ok(v) = opts.get::<_, f32>("volume") {
                                    vol = v;
                                }
                                if let Ok(p) = opts.get::<_, f32>("pitch") {
                                    pitch = p;
                                }
                                if let Ok(l) = opts.get::<_, bool>("loop") {
                                    r#loop = l;
                                }
                            }

                            ec.command_queue
                                .lock()
                                .unwrap()
                                .push(ScriptCommand::PlayAudio {
                                    asset_id: asset_id.clone(),
                                    bus,
                                    volume: vol,
                                    pitch,
                                    r#loop,
                                });

                            format!("audio_{}", rand_id())
                        },
                    )?,
                )?;
            }
            {
                let ec = exec_ctx.clone();
                audio_obj.set(
                    "stop",
                    Function::new(
                        ctx,
                        move |playback_id: Option<String>, bus: Option<String>| {
                            ec.command_queue
                                .lock()
                                .unwrap()
                                .push(ScriptCommand::StopAudio { playback_id, bus });
                        },
                    )?,
                )?;
            }
            {
                let ec = exec_ctx.clone();
                audio_obj.set(
                    "setBusVolume",
                    Function::new(ctx, move |bus: String, volume: f32| {
                        ec.command_queue
                            .lock()
                            .unwrap()
                            .push(ScriptCommand::SetAudioBusVolume { bus, volume });
                    })?,
                )?;
            }
            forge.set("audio", audio_obj)?;

            // 8. forge.animation
            let anim_obj = Object::new(ctx)?;
            {
                let ec = exec_ctx.clone();
                anim_obj.set(
                    "play",
                    Function::new(ctx, move |clip_name: String, entity_id: Option<String>| {
                        ec.command_queue
                            .lock()
                            .unwrap()
                            .push(ScriptCommand::PlayAnimation {
                                clip_name,
                                entity_id,
                            });
                    })?,
                )?;
            }
            forge.set("animation", anim_obj)?;

            // 9. forge.tilemap
            let tilemap_obj = Object::new(ctx)?;
            {
                let ec = exec_ctx.clone();
                tilemap_obj.set(
                    "getTile",
                    Function::new(
                        ctx,
                        move |entity_id: String, layer: String, x: i32, y: i32| -> i32 {
                            let cache = ec.tile_cache.lock().unwrap();
                            cache.get(&(entity_id, layer, x, y)).copied().unwrap_or(-1)
                        },
                    )?,
                )?;
            }
            {
                let ec = exec_ctx.clone();
                tilemap_obj.set(
                    "setTile",
                    Function::new(
                        ctx,
                        move |entity_id: String, layer: String, x: i32, y: i32, tile_id: i32| {
                            ec.tile_cache
                                .lock()
                                .unwrap()
                                .insert((entity_id.clone(), layer.clone(), x, y), tile_id);
                            ec.command_queue
                                .lock()
                                .unwrap()
                                .push(ScriptCommand::SetTile {
                                    entity_id,
                                    layer,
                                    coord: [x, y],
                                    tile_id,
                                });
                        },
                    )?,
                )?;
            }
            forge.set("tilemap", tilemap_obj)?;

            // 10. forge.log
            let log_obj = Object::new(ctx)?;
            log_obj.set(
                "info",
                Function::new(ctx, |msg: Value<'_>| {
                    tracing::info!("[script] {}", msg.to_string().unwrap_or_default());
                })?,
            )?;
            log_obj.set(
                "warn",
                Function::new(ctx, |msg: Value<'_>| {
                    tracing::warn!("[script] {}", msg.to_string().unwrap_or_default());
                })?,
            )?;
            log_obj.set(
                "error",
                Function::new(ctx, |msg: Value<'_>| {
                    tracing::error!("[script] {}", msg.to_string().unwrap_or_default());
                })?,
            )?;
            forge.set("log", log_obj)?;

            global.set("forge", forge)?;
            Ok(())
        })
        .map_err(|e: rquickjs::Error| format!("Failed to bind forge API: {}", e))
}

fn rand_id() -> u32 {
    use std::time::SystemTime;
    let n = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    (n % 1_000_000) as u32
}
