//! Protocol-driven scene operations executed directly on the ECS `World`.

use crate::components::{Camera2D, Children, EntityId, Name, Parent, Sprite, Transform2D, Vec2};
use crate::ecs::{Entity, World};
use crate::error::SceneError;
use forge_protocol::{
    EntityNode, SceneCreateEntityParams, SceneCreateEntityResult, SceneDeleteEntityParams,
    SceneDeleteEntityResult, SceneGetTreeParams, SceneGetTreeResult, SceneSetComponentParams,
    SceneSetComponentResult,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static ENTITY_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Finds an ECS `Entity` in the world by its stable `EntityId`.
pub fn find_entity_by_id(world: &World, id: &str) -> Option<Entity> {
    for entity_ref in world.iter_entities() {
        if let Some(entity_id) = entity_ref.get::<EntityId>() {
            if entity_id.as_str() == id {
                return Some(entity_ref.id());
            }
        }
    }
    None
}

/// Recursively builds an `EntityNode` hierarchy tree starting from the given entity.
pub fn build_entity_node(world: &World, entity: Entity) -> EntityNode {
    let id = world
        .get::<EntityId>(entity)
        .map(|e| e.0.clone())
        .unwrap_or_else(|| format!("entity_{}", entity.index()));

    let name = world
        .get::<Name>(entity)
        .map(|n| n.0.clone())
        .unwrap_or_else(|| id.clone());

    let mut components = Vec::new();
    if world.get::<Name>(entity).is_some() {
        components.push("Name".to_string());
    }
    if world.get::<Transform2D>(entity).is_some() {
        components.push("Transform2D".to_string());
    }
    if world.get::<Sprite>(entity).is_some() {
        components.push("Sprite".to_string());
    }
    if world.get::<Camera2D>(entity).is_some() {
        components.push("Camera2D".to_string());
    }
    if world.get::<Parent>(entity).is_some() {
        components.push("Parent".to_string());
    }
    if world.get::<Children>(entity).is_some() {
        components.push("Children".to_string());
    }
    if world.get::<crate::tilemap::Tilemap>(entity).is_some() {
        components.push("Tilemap".to_string());
    }
    components.sort();

    let mut children_nodes = Vec::new();
    if let Some(children) = world.get::<Children>(entity) {
        for &child in children.as_slice() {
            if world.get_entity(child).is_ok() {
                children_nodes.push(build_entity_node(world, child));
            }
        }
    }
    children_nodes.sort_by(|a, b| a.id.cmp(&b.id));

    EntityNode {
        id,
        name,
        components,
        children: children_nodes,
    }
}

/// Retrieves the scene hierarchy tree according to the protocol `scene.get_tree` contract.
pub fn scene_get_tree(
    world: &World,
    params: &SceneGetTreeParams,
) -> Result<SceneGetTreeResult, SceneError> {
    if let Some(ref root_id) = params.root_entity_id {
        let root_entity = find_entity_by_id(world, root_id)
            .ok_or_else(|| SceneError::EntityNotFound(root_id.clone()))?;

        let node = build_entity_node(world, root_entity);
        Ok(SceneGetTreeResult {
            root_entities: vec![node],
        })
    } else {
        // Collect all entities without a Parent
        let mut root_entities = Vec::new();

        for entity_ref in world.iter_entities() {
            if entity_ref.get::<EntityId>().is_some() && entity_ref.get::<Parent>().is_none() {
                root_entities.push(build_entity_node(world, entity_ref.id()));
            }
        }

        root_entities.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(SceneGetTreeResult { root_entities })
    }
}

/// Creates a new entity in the world according to the protocol `scene.create_entity` contract.
pub fn scene_create_entity(
    world: &mut World,
    params: &SceneCreateEntityParams,
) -> Result<SceneCreateEntityResult, SceneError> {
    let parent_entity = if let Some(ref pid) = params.parent_id {
        let parent =
            find_entity_by_id(world, pid).ok_or_else(|| SceneError::EntityNotFound(pid.clone()))?;
        Some(parent)
    } else {
        None
    };

    let new_id = format!(
        "entity_{:04}",
        ENTITY_COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    let mut entity_builder = world.spawn((EntityId::new(&new_id), Name::new(&params.name)));

    if let Some(parent) = parent_entity {
        entity_builder.insert(Parent::new(parent));
    }

    let entity = entity_builder.id();

    // Register child in parent's Children component
    if let Some(parent) = parent_entity {
        if let Some(mut children) = world.get_mut::<Children>(parent) {
            children.add(entity);
        } else {
            let mut children = Children::new();
            children.add(entity);
            world.entity_mut(parent).insert(children);
        }
    }

    // Apply any initial components
    if let Some(ref components) = params.components {
        for (comp_name, comp_data) in components {
            apply_component_from_str_or_json(world, entity, comp_name, comp_data)?;
        }
    }

    Ok(SceneCreateEntityResult { entity_id: new_id })
}

/// Sets or updates a component on an entity according to `scene.set_component`.
pub fn scene_set_component(
    world: &mut World,
    params: &SceneSetComponentParams,
) -> Result<SceneSetComponentResult, SceneError> {
    let entity = find_entity_by_id(world, &params.entity_id)
        .ok_or_else(|| SceneError::EntityNotFound(params.entity_id.clone()))?;

    apply_component_from_hashmap(world, entity, &params.component_name, &params.data)?;

    Ok(SceneSetComponentResult { success: true })
}

/// Deletes an entity and all its descendants according to `scene.delete_entity`.
pub fn scene_delete_entity(
    world: &mut World,
    params: &SceneDeleteEntityParams,
) -> Result<SceneDeleteEntityResult, SceneError> {
    let entity = find_entity_by_id(world, &params.entity_id)
        .ok_or_else(|| SceneError::EntityNotFound(params.entity_id.clone()))?;

    // Collect all descendant entities
    let mut to_despawn = Vec::new();
    collect_descendants(world, entity, &mut to_despawn);
    to_despawn.push(entity);

    // If the target has a parent, remove it from parent's Children
    if let Some(parent) = world.get::<Parent>(entity) {
        let parent_entity = parent.entity();
        if let Some(mut children) = world.get_mut::<Children>(parent_entity) {
            children.remove(entity);
        }
    }

    // Despawn all collected entities
    for e in to_despawn {
        if world.get_entity(e).is_ok() {
            world.despawn(e);
        }
    }

    Ok(SceneDeleteEntityResult { success: true })
}

fn collect_descendants(world: &World, entity: Entity, list: &mut Vec<Entity>) {
    if let Some(children) = world.get::<Children>(entity) {
        for &child in children.as_slice() {
            collect_descendants(world, child, list);
            list.push(child);
        }
    }
}

fn apply_component_from_str_or_json(
    world: &mut World,
    entity: Entity,
    comp_name: &str,
    data_str: &str,
) -> Result<(), SceneError> {
    // Attempt parsing as JSON object first
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(data_str) {
        if let Some(obj) = value.as_object() {
            let mut map = HashMap::new();
            for (k, v) in obj {
                let s = match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                map.insert(k.clone(), s);
            }
            return apply_component_from_hashmap(world, entity, comp_name, &map);
        }
    }

    let mut map = HashMap::new();
    map.insert("value".to_string(), data_str.to_string());
    apply_component_from_hashmap(world, entity, comp_name, &map)
}

fn apply_component_from_hashmap(
    world: &mut World,
    entity: Entity,
    comp_name: &str,
    data: &HashMap<String, String>,
) -> Result<(), SceneError> {
    match comp_name {
        "Name" => {
            let name_str = data
                .get("name")
                .or_else(|| data.get("value"))
                .cloned()
                .unwrap_or_else(|| "Entity".to_string());
            world.entity_mut(entity).insert(Name::new(name_str));
            Ok(())
        }
        "Transform" | "Transform2D" => {
            let mut transform = world
                .get::<Transform2D>(entity)
                .copied()
                .unwrap_or_default();

            if let Some(pos_str) = data.get("position") {
                if let Ok(coords) = serde_json::from_str::<Vec<f32>>(pos_str) {
                    if coords.len() >= 2 {
                        transform.position = Vec2::new(coords[0], coords[1]);
                    }
                } else if let Ok(val) = serde_json::from_str::<serde_json::Value>(pos_str) {
                    if let (Some(x), Some(y)) = (
                        val.get("x").and_then(|v| v.as_f64()),
                        val.get("y").and_then(|v| v.as_f64()),
                    ) {
                        transform.position = Vec2::new(x as f32, y as f32);
                    }
                }
            }
            if let Some(x_str) = data.get("x") {
                if let Ok(x) = x_str.parse::<f32>() {
                    transform.position.x = x;
                }
            }
            if let Some(y_str) = data.get("y") {
                if let Ok(y) = y_str.parse::<f32>() {
                    transform.position.y = y;
                }
            }
            if let Some(rot_str) = data.get("rotation") {
                if let Ok(r) = rot_str.parse::<f32>() {
                    transform.rotation = r;
                } else if let Ok(coords) = serde_json::from_str::<Vec<f32>>(rot_str) {
                    if coords.len() >= 4 {
                        transform.rotation = coords[2]; // Use z rotation if quaternion
                    }
                }
            }
            if let Some(scale_str) = data.get("scale") {
                if let Ok(s) = scale_str.parse::<f32>() {
                    transform.scale = Vec2::new(s, s);
                } else if let Ok(coords) = serde_json::from_str::<Vec<f32>>(scale_str) {
                    if coords.len() >= 2 {
                        transform.scale = Vec2::new(coords[0], coords[1]);
                    }
                }
            }

            world.entity_mut(entity).insert(transform);
            Ok(())
        }
        "Sprite" => {
            let mut sprite = world.get::<Sprite>(entity).cloned().unwrap_or_default();

            if let Some(asset_id) = data.get("asset_id") {
                sprite.asset_id = asset_id.clone();
            }
            if let Some(size_str) = data.get("size") {
                if let Ok(dims) = serde_json::from_str::<Vec<f32>>(size_str) {
                    if dims.len() >= 2 {
                        sprite.size = Vec2::new(dims[0], dims[1]);
                    }
                } else if let Ok(val) = serde_json::from_str::<serde_json::Value>(size_str) {
                    if let (Some(w), Some(h)) = (
                        val.get("x").and_then(|v| v.as_f64()),
                        val.get("y").and_then(|v| v.as_f64()),
                    ) {
                        sprite.size = Vec2::new(w as f32, h as f32);
                    }
                }
            }
            if let Some(tint_str) = data.get("color_tint") {
                if let Ok(tint) = serde_json::from_str::<[f32; 4]>(tint_str) {
                    sprite.color_tint = tint;
                }
            }
            if let Some(z_str) = data.get("z_order") {
                if let Ok(z) = z_str.parse::<i32>() {
                    sprite.z_order = z;
                }
            }
            if let Some(frame_str) = data.get("frame") {
                if let Ok(f) = frame_str.parse::<u32>() {
                    sprite.frame = Some(f);
                }
            }
            if let Some(uv_str) = data.get("uv_rect") {
                if let Ok(uv) = serde_json::from_str::<[f32; 4]>(uv_str) {
                    sprite.uv_rect = Some(uv);
                }
            }

            world.entity_mut(entity).insert(sprite);
            Ok(())
        }
        "Animator" => {
            let mut animator = world
                .get::<crate::components::Animator>(entity)
                .cloned()
                .unwrap_or_default();

            if let Some(clip_id) = data.get("clip_asset_id").or_else(|| data.get("clip")) {
                animator.clip_asset_id = clip_id.clone();
            }
            if let Some(playing_str) = data.get("playing") {
                if let Ok(p) = playing_str.parse::<bool>() {
                    animator.playing = p;
                }
            }
            if let Some(speed_str) = data.get("speed") {
                if let Ok(s) = speed_str.parse::<f32>() {
                    animator.speed = s;
                }
            }

            world.entity_mut(entity).insert(animator);
            Ok(())
        }
        "Camera" | "Camera2D" => {
            let mut camera = world.get::<Camera2D>(entity).cloned().unwrap_or_default();

            if let Some(zoom_str) = data.get("zoom") {
                if let Ok(zoom) = zoom_str.parse::<f32>() {
                    camera.zoom = zoom;
                }
            }
            if let Some(active_str) = data.get("is_active") {
                if let Ok(active) = active_str.parse::<bool>() {
                    camera.is_active = active;
                }
            }

            world.entity_mut(entity).insert(camera);
            Ok(())
        }
        "RigidBody" | "RigidBody2D" => {
            let mut rb = world
                .get::<crate::components::RigidBody2D>(entity)
                .cloned()
                .unwrap_or_default();

            if let Some(body_type_str) = data.get("body_type") {
                if let Ok(bt) = serde_json::from_str::<crate::components::BodyType2D>(body_type_str)
                {
                    rb.body_type = bt;
                } else if body_type_str.eq_ignore_ascii_case("dynamic") {
                    rb.body_type = crate::components::BodyType2D::Dynamic;
                } else if body_type_str.eq_ignore_ascii_case("kinematic") {
                    rb.body_type = crate::components::BodyType2D::Kinematic;
                } else if body_type_str.eq_ignore_ascii_case("static") {
                    rb.body_type = crate::components::BodyType2D::Static;
                }
            }
            if let Some(grav_str) = data.get("gravity_scale") {
                if let Ok(g) = grav_str.parse::<f32>() {
                    rb.gravity_scale = g;
                }
            }
            if let Some(lock_rot_str) = data.get("lock_rotation") {
                if let Ok(lr) = lock_rot_str.parse::<bool>() {
                    rb.lock_rotation = lr;
                }
            }
            if let Some(damping_str) = data.get("linear_damping") {
                if let Ok(ld) = damping_str.parse::<f32>() {
                    rb.linear_damping = ld;
                }
            }
            if let Some(ang_damping_str) = data.get("angular_damping") {
                if let Ok(ad) = ang_damping_str.parse::<f32>() {
                    rb.angular_damping = ad;
                }
            }

            world.entity_mut(entity).insert(rb);
            Ok(())
        }
        "Collider" | "Collider2D" => {
            let mut col = world
                .get::<crate::components::Collider2D>(entity)
                .cloned()
                .unwrap_or_default();

            if let Some(shape_str) = data.get("shape") {
                if let Ok(s) = serde_json::from_str::<crate::components::ColliderShape2D>(shape_str)
                {
                    col.shape = s;
                }
            }
            if let Some(sensor_str) = data.get("is_sensor") {
                if let Ok(s) = sensor_str.parse::<bool>() {
                    col.is_sensor = s;
                }
            }
            if let Some(fric_str) = data.get("friction") {
                if let Ok(f) = fric_str.parse::<f32>() {
                    col.friction = f;
                }
            }
            if let Some(rest_str) = data.get("restitution") {
                if let Ok(r) = rest_str.parse::<f32>() {
                    col.restitution = r;
                }
            }
            if let Some(density_str) = data.get("density") {
                if let Ok(d) = density_str.parse::<f32>() {
                    col.density = d;
                }
            }

            world.entity_mut(entity).insert(col);
            Ok(())
        }
        "Tilemap" => {
            let mut tm = world
                .get::<crate::tilemap::Tilemap>(entity)
                .cloned()
                .unwrap_or_default();

            if let Some(tileset_id) = data.get("tileset_id") {
                tm.tileset_id = tileset_id.clone();
            }
            if let Some(cs_str) = data.get("chunk_size") {
                if let Ok(cs) = cs_str.parse::<u32>() {
                    tm.chunk_size = cs;
                }
            }
            if let Some(z_str) = data.get("z_order") {
                if let Ok(z) = z_str.parse::<i32>() {
                    tm.z_order = z;
                }
            }
            world.entity_mut(entity).insert(tm);
            Ok(())
        }
        other => Err(SceneError::UnknownComponent(other.to_string())),
    }
}

/// Extension trait providing scene protocol operations directly on the ECS `World`.
pub trait WorldSceneExt {
    /// Retrieves the scene tree.
    fn scene_get_tree(&self, params: &SceneGetTreeParams)
        -> Result<SceneGetTreeResult, SceneError>;
    /// Creates an entity.
    fn scene_create_entity(
        &mut self,
        params: &SceneCreateEntityParams,
    ) -> Result<SceneCreateEntityResult, SceneError>;
    /// Sets a component on an entity.
    fn scene_set_component(
        &mut self,
        params: &SceneSetComponentParams,
    ) -> Result<SceneSetComponentResult, SceneError>;
    /// Deletes an entity.
    fn scene_delete_entity(
        &mut self,
        params: &SceneDeleteEntityParams,
    ) -> Result<SceneDeleteEntityResult, SceneError>;
}

impl WorldSceneExt for World {
    fn scene_get_tree(
        &self,
        params: &SceneGetTreeParams,
    ) -> Result<SceneGetTreeResult, SceneError> {
        scene_get_tree(self, params)
    }

    fn scene_create_entity(
        &mut self,
        params: &SceneCreateEntityParams,
    ) -> Result<SceneCreateEntityResult, SceneError> {
        scene_create_entity(self, params)
    }

    fn scene_set_component(
        &mut self,
        params: &SceneSetComponentParams,
    ) -> Result<SceneSetComponentResult, SceneError> {
        scene_set_component(self, params)
    }

    fn scene_delete_entity(
        &mut self,
        params: &SceneDeleteEntityParams,
    ) -> Result<SceneDeleteEntityResult, SceneError> {
        scene_delete_entity(self, params)
    }
}
