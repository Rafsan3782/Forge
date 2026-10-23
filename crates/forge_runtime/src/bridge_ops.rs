//! Bridge command execution between frames on the engine thread.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use forge_assets::TextureStore;
use forge_bridge::{BridgeCommand, BridgeCommandReceiver};
use forge_core::scene_ops::{
    scene_create_entity, scene_delete_entity, scene_get_tree, scene_set_component,
};
use forge_core::World;
use forge_protocol::{
    EnginePingResult, GameScreenshotParams, GameScreenshotResult, ProjectStopResult,
};
use forge_render::{OffscreenTarget, RenderContext, Renderer2D};
use image::ImageEncoder;

use crate::render_scene::{render_world_scene, DEFAULT_CLEAR_COLOR};

/// Helper to get current timestamp in milliseconds.
pub fn current_timestamp_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
}

/// Drains pending bridge commands from the queue without holding locks across frames.
#[allow(clippy::too_many_arguments)]
pub fn process_bridge_commands(
    command_rx: &mut BridgeCommandReceiver,
    world: &mut World,
    renderer: &mut Renderer2D,
    texture_store: &mut TextureStore,
    context: &RenderContext,
    default_width: u32,
    default_height: u32,
    shutdown_requested: &mut bool,
) {
    while let Ok(cmd) = command_rx.try_recv() {
        match cmd {
            BridgeCommand::EnginePing { params, responder } => {
                let timestamp = if params.timestamp != 0.0 {
                    params.timestamp
                } else {
                    current_timestamp_ms()
                };
                let _ = responder.send(Ok(EnginePingResult {
                    status: "pong".to_string(),
                    timestamp,
                }));
            }

            BridgeCommand::SceneGetTree { params, responder } => {
                let res = scene_get_tree(world, &params).map_err(|e| e.to_string());
                let _ = responder.send(res);
            }

            BridgeCommand::SceneCreateEntity { params, responder } => {
                let res = scene_create_entity(world, &params).map_err(|e| e.to_string());
                let _ = responder.send(res);
            }

            BridgeCommand::SceneSetComponent { params, responder } => {
                let res = scene_set_component(world, &params).map_err(|e| e.to_string());
                let _ = responder.send(res);
            }

            BridgeCommand::SceneDeleteEntity { params, responder } => {
                let res = scene_delete_entity(world, &params).map_err(|e| e.to_string());
                let _ = responder.send(res);
            }

            BridgeCommand::GameScreenshot { params, responder } => {
                let res = capture_screenshot(
                    world,
                    renderer,
                    texture_store,
                    context,
                    &params,
                    default_width,
                    default_height,
                );
                let _ = responder.send(res);
            }

            BridgeCommand::GameSimulateInput { params, responder } => {
                let res = apply_simulated_input(world, &params);
                let _ = responder.send(Ok(res));
            }

            BridgeCommand::ProjectStop {
                params: _,
                responder,
            } => {
                *shutdown_requested = true;
                let _ = responder.send(Ok(ProjectStopResult { stopped: true }));
            }

            BridgeCommand::AssetList { params, responder } => {
                let res = handle_asset_list(world, &params);
                let _ = responder.send(res);
            }

            BridgeCommand::AssetGetInfo { params, responder } => {
                let res = handle_asset_get_info(world, &params);
                let _ = responder.send(res);
            }

            BridgeCommand::AssetImport { params, responder } => {
                let res = handle_asset_import(world, &params);
                let _ = responder.send(res);
            }

            BridgeCommand::PhysicsRaycast { params, responder } => {
                let res = handle_physics_raycast(world, &params);
                let _ = responder.send(res);
            }

            BridgeCommand::TilemapGet { params, responder } => {
                let res = handle_tilemap_get(world, &params);
                let _ = responder.send(res);
            }

            BridgeCommand::TilemapSetTile { params, responder } => {
                let res = handle_tilemap_set_tile(world, &params);
                let _ = responder.send(res);
            }

            BridgeCommand::TilemapFillRect { params, responder } => {
                let res = handle_tilemap_fill_rect(world, &params);
                let _ = responder.send(res);
            }

            BridgeCommand::TilemapClear { params, responder } => {
                let res = handle_tilemap_clear(world, &params);
                let _ = responder.send(res);
            }
        }
    }
}

fn handle_asset_list(
    world: &World,
    params: &forge_protocol::AssetListParams,
) -> Result<forge_protocol::AssetListResult, String> {
    let db = world
        .get_resource::<forge_assets::AssetDatabase>()
        .ok_or_else(|| "AssetDatabase resource not initialized in world".to_string())?;

    let filter_type = params.asset_type.as_deref();
    let mut assets = Vec::new();

    for entry in db.all_entries() {
        let type_str = match &entry.meta.asset_type {
            forge_assets::AssetType::Texture => "texture".to_string(),
            forge_assets::AssetType::SpriteSheet => "spritesheet".to_string(),
            forge_assets::AssetType::Animation => "animation".to_string(),
            forge_assets::AssetType::Scene => "scene".to_string(),
            forge_assets::AssetType::Audio => "audio".to_string(),
            forge_assets::AssetType::Tileset => "tileset".to_string(),
            forge_assets::AssetType::Other(s) => s.clone(),
        };

        if let Some(ft) = filter_type {
            if !ft.is_empty() && !type_str.eq_ignore_ascii_case(ft) {
                continue;
            }
        }

        assets.push(forge_protocol::AssetInfo {
            id: entry.id.clone(),
            path: entry.file_path.to_string_lossy().to_string(),
            asset_type: type_str,
            metadata: entry.meta.import_settings.clone(),
        });
    }

    Ok(forge_protocol::AssetListResult { assets })
}

fn handle_asset_get_info(
    world: &World,
    params: &forge_protocol::AssetGetInfoParams,
) -> Result<forge_protocol::AssetGetInfoResult, String> {
    let db = world
        .get_resource::<forge_assets::AssetDatabase>()
        .ok_or_else(|| "AssetDatabase resource not initialized in world".to_string())?;

    let entry = db
        .get(&params.id)
        .ok_or_else(|| format!("Asset with id '{}' not found", params.id))?;

    let type_str = match &entry.meta.asset_type {
        forge_assets::AssetType::Texture => "texture".to_string(),
        forge_assets::AssetType::SpriteSheet => "spritesheet".to_string(),
        forge_assets::AssetType::Animation => "animation".to_string(),
        forge_assets::AssetType::Scene => "scene".to_string(),
        forge_assets::AssetType::Audio => "audio".to_string(),
        forge_assets::AssetType::Tileset => "tileset".to_string(),
        forge_assets::AssetType::Other(s) => s.clone(),
    };

    Ok(forge_protocol::AssetGetInfoResult {
        id: entry.id.clone(),
        path: entry.file_path.to_string_lossy().to_string(),
        asset_type: type_str,
        metadata: entry.meta.import_settings.clone(),
    })
}

fn handle_asset_import(
    world: &mut World,
    params: &forge_protocol::AssetImportParams,
) -> Result<forge_protocol::AssetImportResult, String> {
    let mut db = world
        .get_resource_mut::<forge_assets::AssetDatabase>()
        .ok_or_else(|| "AssetDatabase resource not initialized in world".to_string())?;

    let source_path = std::path::Path::new(&params.source_path);
    let target_path = std::path::Path::new(&params.target_path);

    let explicit_type = params
        .asset_type
        .as_deref()
        .map(|t| match t.to_lowercase().as_str() {
            "texture" => forge_assets::AssetType::Texture,
            "spritesheet" => forge_assets::AssetType::SpriteSheet,
            "animation" => forge_assets::AssetType::Animation,
            "scene" => forge_assets::AssetType::Scene,
            "audio" => forge_assets::AssetType::Audio,
            "tileset" => forge_assets::AssetType::Tileset,
            other => forge_assets::AssetType::Other(other.to_string()),
        });

    let entry = db
        .import_file(
            source_path,
            target_path,
            explicit_type,
            params.metadata.clone(),
        )
        .map_err(|e| format!("Failed to import asset: {}", e))?;

    let type_str = match &entry.meta.asset_type {
        forge_assets::AssetType::Texture => "texture".to_string(),
        forge_assets::AssetType::SpriteSheet => "spritesheet".to_string(),
        forge_assets::AssetType::Animation => "animation".to_string(),
        forge_assets::AssetType::Scene => "scene".to_string(),
        forge_assets::AssetType::Audio => "audio".to_string(),
        forge_assets::AssetType::Tileset => "tileset".to_string(),
        forge_assets::AssetType::Other(s) => s.clone(),
    };

    Ok(forge_protocol::AssetImportResult {
        id: entry.id,
        path: entry.file_path.to_string_lossy().to_string(),
        asset_type: type_str,
    })
}

/// Renders the world into an offscreen target and produces a PNG screenshot (base64 or file).
#[allow(clippy::too_many_arguments)]
pub fn capture_screenshot(
    world: &World,
    renderer: &mut Renderer2D,
    texture_store: &mut TextureStore,
    context: &RenderContext,
    params: &GameScreenshotParams,
    default_width: u32,
    default_height: u32,
) -> Result<GameScreenshotResult, String> {
    let width = params.width.unwrap_or(default_width).max(1);
    let height = params.height.unwrap_or(default_height).max(1);

    let offscreen = OffscreenTarget::new(&context.device, width, height)
        .map_err(|e| format!("Failed to create offscreen render target: {}", e))?;

    render_world_scene(
        world,
        renderer,
        texture_store,
        context,
        &offscreen.view,
        offscreen.format,
        width as f32,
        height as f32,
        Some(DEFAULT_CLEAR_COLOR),
    )
    .map_err(|e| format!("Failed to render scene for screenshot: {}", e))?;

    let rgba_bytes = offscreen
        .read_rgba(&context.device, &context.queue)
        .map_err(|e| format!("Failed to readback screenshot buffer: {}", e))?;

    let mut png_bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    encoder
        .write_image(&rgba_bytes, width, height, image::ExtendedColorType::Rgba8)
        .map_err(|e| format!("Failed to encode PNG screenshot: {}", e))?;

    let file_path = if let Some(ref path_str) = params.file_path {
        let path = Path::new(path_str);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory for screenshot: {}", e))?;
            }
        }
        std::fs::write(path, &png_bytes)
            .map_err(|e| format!("Failed to write screenshot to '{}': {}", path_str, e))?;
        Some(path_str.clone())
    } else {
        None
    };

    let data_base64 = if file_path.is_none() {
        Some(base64::engine::general_purpose::STANDARD.encode(&png_bytes))
    } else {
        None
    };

    Ok(GameScreenshotResult {
        data_base64,
        file_path,
        width,
        height,
        format: "png".to_string(),
    })
}

/// Applies simulated input events to the World's InputState and records into InputRingBuffer.
pub fn apply_simulated_input(
    world: &mut World,
    params: &forge_protocol::GameSimulateInputParams,
) -> forge_protocol::GameSimulateInputResult {
    use forge_input::{InputRingBuffer, InputState, KeyCode, MouseButton};

    let mut applied_count = 0;

    if let Some(mut state) = world.get_resource_mut::<InputState>() {
        let frame = state.current_frame();
        for ev in &params.events {
            applied_count += 1;
            match ev.input_type.as_str() {
                "key_down" => {
                    if let Some(ref k_str) = ev.key {
                        if let Some(k) = KeyCode::from_name(k_str) {
                            state.simulate_key(k, true, ev.duration_frames);
                        }
                    }
                }
                "key_up" => {
                    if let Some(ref k_str) = ev.key {
                        if let Some(k) = KeyCode::from_name(k_str) {
                            state.simulate_key(k, false, 1);
                        }
                    }
                }
                "action_down" => {
                    if let Some(ref act) = ev.action {
                        state.simulate_action(act, true, ev.duration_frames);
                    }
                }
                "action_up" => {
                    if let Some(ref act) = ev.action {
                        state.simulate_action(act, false, 1);
                    }
                }
                "action_axis" => {
                    if let Some(ref act) = ev.action {
                        let val = ev.value.unwrap_or(0.0) as f32;
                        state.simulate_axis(act, val, ev.duration_frames);
                    }
                }
                "mouse_move" => {
                    let val = ev.value.unwrap_or(0.0) as f32;
                    state.feed_mouse_move(forge_core::Vec2::new(val, val), None);
                }
                "mouse_down" => {
                    let btn = ev
                        .key
                        .as_deref()
                        .and_then(MouseButton::from_name)
                        .unwrap_or(MouseButton::Left);
                    state.feed_mouse_button_press(btn, None);
                }
                "mouse_up" => {
                    let btn = ev
                        .key
                        .as_deref()
                        .and_then(MouseButton::from_name)
                        .unwrap_or(MouseButton::Left);
                    state.feed_mouse_button_release(btn, None);
                }
                _ => {}
            }
        }

        // Also record to ring buffer if present
        if let Some(mut buf) = world.get_resource_mut::<InputRingBuffer>() {
            for ev in &params.events {
                buf.record(
                    frame,
                    ev.input_type.clone(),
                    ev.action
                        .clone()
                        .or_else(|| ev.key.clone())
                        .unwrap_or_default(),
                    ev.value.unwrap_or(1.0) as f32,
                );
            }
        }
    }

    forge_protocol::GameSimulateInputResult {
        success: true,
        applied_count,
    }
}

/// Executes a spatial raycast query against the physics world.
pub fn handle_physics_raycast(
    world: &World,
    params: &forge_protocol::PhysicsRaycastParams,
) -> Result<forge_protocol::PhysicsRaycastResult, String> {
    let physics = world
        .get_resource::<forge_physics::PhysicsWorld2D>()
        .ok_or_else(|| "PhysicsWorld2D resource not initialized in world".to_string())?;

    let origin = forge_core::Vec2::new(params.origin[0] as f32, params.origin[1] as f32);
    let direction = forge_core::Vec2::new(params.direction[0] as f32, params.direction[1] as f32);
    let max_dist = params.max_distance as f32;

    let mut filter = forge_physics::QueryFilter::new();
    if let Some(mask) = params.layer_mask {
        filter = filter.with_mask(mask);
    }
    // Solid flag determines whether non-sensors are queried
    filter = filter.with_sensors(!params.solid);

    if let Some(hit) = physics.cast_ray(origin, direction, max_dist, filter) {
        let entity_id_str = world
            .get::<forge_core::EntityId>(hit.entity)
            .map(|id| id.as_str().to_string())
            .unwrap_or_else(|| format!("entity_{:?}", hit.entity));

        Ok(forge_protocol::PhysicsRaycastResult {
            hit: true,
            data: Some(forge_protocol::RaycastHit {
                entity_id: entity_id_str,
                point: [hit.point.x as f64, hit.point.y as f64],
                normal: [hit.normal.x as f64, hit.normal.y as f64],
                fraction: hit.fraction as f64,
            }),
        })
    } else {
        Ok(forge_protocol::PhysicsRaycastResult {
            hit: false,
            data: None,
        })
    }
}

fn find_tilemap_entity(
    world: &World,
    entity_id_str: &str,
) -> Result<forge_core::ecs::Entity, String> {
    for entity_ref in world.iter_entities() {
        if let Some(id) = entity_ref.get::<forge_core::EntityId>() {
            if id.as_str() == entity_id_str {
                return Ok(entity_ref.id());
            }
        }
    }
    Err(format!("Entity with id '{}' not found", entity_id_str))
}

fn handle_tilemap_get(
    world: &World,
    params: &forge_protocol::TilemapGetParams,
) -> Result<forge_protocol::TilemapGetResult, String> {
    let entity = find_tilemap_entity(world, &params.entity_id)?;
    let tilemap = world.get::<forge_core::Tilemap>(entity).ok_or_else(|| {
        format!(
            "Entity '{}' does not have a Tilemap component",
            params.entity_id
        )
    })?;

    let layer = params.layer.as_deref().unwrap_or("main");

    let tile_id = if let Some(tc) = params.tile_coord {
        Some(tilemap.get_tile(layer, tc[0], tc[1]))
    } else {
        None
    };

    let chunk_data = if let Some(cc) = params.chunk_coord {
        tilemap
            .chunks
            .get(&cc)
            .and_then(|chunk| chunk.layers.get(layer).map(|rows| rows.join(";")))
    } else {
        None
    };

    Ok(forge_protocol::TilemapGetResult {
        tileset_id: tilemap.tileset_id.clone(),
        tile_size: [tilemap.tile_size.x, tilemap.tile_size.y],
        chunk_size: tilemap.chunk_size,
        layers: tilemap.layers.clone(),
        tile_id,
        chunk_data,
    })
}

fn handle_tilemap_set_tile(
    world: &mut World,
    params: &forge_protocol::TilemapSetTileParams,
) -> Result<forge_protocol::TilemapSetTileResult, String> {
    let entity = find_tilemap_entity(world, &params.entity_id)?;
    let (prev_tile_id, chunk_coord) = {
        let mut tilemap = world
            .get_mut::<forge_core::Tilemap>(entity)
            .ok_or_else(|| {
                format!(
                    "Entity '{}' does not have a Tilemap component",
                    params.entity_id
                )
            })?;

        let layer = if params.layer.is_empty() {
            "main"
        } else {
            &params.layer
        };
        let prev = tilemap.set_tile(layer, params.coord[0], params.coord[1], params.tile_id);
        let (coord, _) = tilemap.tile_to_chunk_coord(params.coord[0], params.coord[1]);
        (prev, coord)
    };

    // Mark dirty in collider cache if present
    if let Some(mut cache) = world.get_mut::<forge_physics::TilemapColliderCache>(entity) {
        cache.mark_dirty(chunk_coord);
    }

    Ok(forge_protocol::TilemapSetTileResult {
        success: true,
        prev_tile_id,
        chunk_coord: Some(chunk_coord),
    })
}

fn handle_tilemap_fill_rect(
    world: &mut World,
    params: &forge_protocol::TilemapFillRectParams,
) -> Result<forge_protocol::TilemapFillRectResult, String> {
    let entity = find_tilemap_entity(world, &params.entity_id)?;
    let (tiles_modified, affected_chunks) = {
        let mut tilemap = world
            .get_mut::<forge_core::Tilemap>(entity)
            .ok_or_else(|| {
                format!(
                    "Entity '{}' does not have a Tilemap component",
                    params.entity_id
                )
            })?;

        let layer = if params.layer.is_empty() {
            "main"
        } else {
            &params.layer
        };
        tilemap.fill_rect(layer, params.min, params.max, params.tile_id)
    };

    // Mark all chunks dirty in collider cache if present
    if let Some(mut cache) = world.get_mut::<forge_physics::TilemapColliderCache>(entity) {
        cache.chunk_colliders.clear();
    }

    Ok(forge_protocol::TilemapFillRectResult {
        success: true,
        tiles_modified,
        affected_chunks,
    })
}

fn handle_tilemap_clear(
    world: &mut World,
    params: &forge_protocol::TilemapClearParams,
) -> Result<forge_protocol::TilemapClearResult, String> {
    let entity = find_tilemap_entity(world, &params.entity_id)?;
    let chunks_cleared = {
        let mut tilemap = world
            .get_mut::<forge_core::Tilemap>(entity)
            .ok_or_else(|| {
                format!(
                    "Entity '{}' does not have a Tilemap component",
                    params.entity_id
                )
            })?;

        tilemap.clear_map(params.layer.as_deref())
    };

    // Clear collider cache
    if let Some(mut cache) = world.get_mut::<forge_physics::TilemapColliderCache>(entity) {
        cache.chunk_colliders.clear();
        cache.dirty_chunks.clear();
    }

    Ok(forge_protocol::TilemapClearResult {
        success: true,
        chunks_cleared,
    })
}
