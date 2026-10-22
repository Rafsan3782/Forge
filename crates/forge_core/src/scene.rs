//! Scene data model, stable serialization, and world loading/saving.

use crate::components::{
    Camera2D, Children, Collider2D, EntityId, Name, Parent, RigidBody2D, Sprite, Transform2D,
};
use crate::ecs::{Entity, World};
use crate::error::SceneError;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

/// Serialized representation of a single entity within a scene.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneEntity {
    /// Stable entity identifier.
    pub id: String,
    /// Stable identifier of the parent entity, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Map of component names to their serialized JSON representations.
    /// Uses BTreeMap for deterministic diff-friendly key sorting.
    pub components: BTreeMap<String, serde_json::Value>,
}

/// A scene containing a named collection of entities and their components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    /// Name of the scene.
    pub name: String,
    /// List of entities belonging to the scene.
    pub entities: Vec<SceneEntity>,
}

impl Scene {
    /// Creates a new scene with the given name and empty entity list.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entities: Vec::new(),
        }
    }

    /// Serializes the scene into a formatted, human-readable, diff-friendly JSON string.
    pub fn to_json_string(&self) -> Result<String, SceneError> {
        let mut sorted_scene = self.clone();
        sorted_scene.entities.sort_by(|a, b| a.id.cmp(&b.id));
        serde_json::to_string_pretty(&sorted_scene).map_err(Into::into)
    }

    /// Deserializes a scene from a JSON string.
    pub fn from_json_str(json: &str) -> Result<Self, SceneError> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Loads a scene from a JSON file on disk.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, SceneError> {
        let content = fs::read_to_string(path)?;
        Self::from_json_str(&content)
    }

    /// Saves the scene to a JSON file on disk.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), SceneError> {
        let json = self.to_json_string()?;
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, json)?;
        Ok(())
    }

    /// Spawns all entities defined in this scene into the given ECS `World`.
    ///
    /// Restores stable entity IDs, built-in components, and Parent/Children hierarchies.
    /// Returns a map of stable entity ID to runtime ECS `Entity`.
    pub fn spawn_into_world(
        &self,
        world: &mut World,
    ) -> Result<HashMap<String, Entity>, SceneError> {
        let mut id_to_entity: HashMap<String, Entity> = HashMap::new();

        // Check for duplicate entity IDs in scene definition
        for se in &self.entities {
            if id_to_entity.contains_key(&se.id) {
                return Err(SceneError::DuplicateEntityId(se.id.clone()));
            }
            // Temporarily insert a placeholder
            id_to_entity.insert(se.id.clone(), Entity::PLACEHOLDER);
        }

        // Pass 1: Spawn entities with components (excluding hierarchy)
        for se in &self.entities {
            let mut entity_builder = world.spawn(EntityId::new(&se.id));

            if let Some(val) = se.components.get("Name") {
                let name: Name = serde_json::from_value(val.clone()).map_err(|e| {
                    SceneError::ComponentParseError {
                        component_name: "Name".to_string(),
                        message: e.to_string(),
                    }
                })?;
                entity_builder.insert(name);
            }

            if let Some(val) = se.components.get("Transform2D") {
                let transform: Transform2D = serde_json::from_value(val.clone()).map_err(|e| {
                    SceneError::ComponentParseError {
                        component_name: "Transform2D".to_string(),
                        message: e.to_string(),
                    }
                })?;
                entity_builder.insert(transform);
            }

            if let Some(val) = se.components.get("Sprite") {
                let sprite: Sprite = serde_json::from_value(val.clone()).map_err(|e| {
                    SceneError::ComponentParseError {
                        component_name: "Sprite".to_string(),
                        message: e.to_string(),
                    }
                })?;
                entity_builder.insert(sprite);
            }

            if let Some(val) = se.components.get("Camera2D") {
                let camera: Camera2D = serde_json::from_value(val.clone()).map_err(|e| {
                    SceneError::ComponentParseError {
                        component_name: "Camera2D".to_string(),
                        message: e.to_string(),
                    }
                })?;
                entity_builder.insert(camera);
            }

            if let Some(val) = se.components.get("RigidBody2D") {
                let rb: RigidBody2D = serde_json::from_value(val.clone()).map_err(|e| {
                    SceneError::ComponentParseError {
                        component_name: "RigidBody2D".to_string(),
                        message: e.to_string(),
                    }
                })?;
                entity_builder.insert(rb);
            }

            if let Some(val) = se.components.get("Collider2D") {
                let col: Collider2D = serde_json::from_value(val.clone()).map_err(|e| {
                    SceneError::ComponentParseError {
                        component_name: "Collider2D".to_string(),
                        message: e.to_string(),
                    }
                })?;
                entity_builder.insert(col);
            }

            let entity = entity_builder.id();
            id_to_entity.insert(se.id.clone(), entity);
        }

        // Pass 2: Establish Parent and Children relationships
        for se in &self.entities {
            if let Some(ref parent_id) = se.parent {
                let parent_entity = id_to_entity
                    .get(parent_id)
                    .copied()
                    .ok_or_else(|| SceneError::EntityNotFound(parent_id.clone()))?;

                let child_entity = id_to_entity[&se.id];

                // Set Parent on child
                world
                    .entity_mut(child_entity)
                    .insert(Parent::new(parent_entity));

                // Add to Children on parent
                if let Some(mut children) = world.get_mut::<Children>(parent_entity) {
                    children.add(child_entity);
                } else {
                    let mut children = Children::new();
                    children.add(child_entity);
                    world.entity_mut(parent_entity).insert(children);
                }
            }
        }

        Ok(id_to_entity)
    }

    /// Extracts a `Scene` model from an existing ECS `World`.
    pub fn from_world(world: &World, scene_name: impl Into<String>) -> Result<Self, SceneError> {
        let mut scene = Self::new(scene_name);

        for entity_ref in world.iter_entities() {
            if let Some(entity_id) = entity_ref.get::<EntityId>() {
                let mut components = BTreeMap::new();

                if let Some(name) = entity_ref.get::<Name>() {
                    components.insert("Name".to_string(), serde_json::to_value(name)?);
                }
                if let Some(transform) = entity_ref.get::<Transform2D>() {
                    components.insert("Transform2D".to_string(), serde_json::to_value(transform)?);
                }
                if let Some(sprite) = entity_ref.get::<Sprite>() {
                    components.insert("Sprite".to_string(), serde_json::to_value(sprite)?);
                }
                if let Some(camera) = entity_ref.get::<Camera2D>() {
                    components.insert("Camera2D".to_string(), serde_json::to_value(camera)?);
                }
                if let Some(rb) = entity_ref.get::<RigidBody2D>() {
                    components.insert("RigidBody2D".to_string(), serde_json::to_value(rb)?);
                }
                if let Some(col) = entity_ref.get::<Collider2D>() {
                    components.insert("Collider2D".to_string(), serde_json::to_value(col)?);
                }

                // Resolve parent stable id
                let parent_id = if let Some(parent) = entity_ref.get::<Parent>() {
                    world
                        .get_entity(parent.entity())
                        .ok()
                        .and_then(|p| p.get::<EntityId>().map(|pid| pid.0.clone()))
                } else {
                    None
                };

                scene.entities.push(SceneEntity {
                    id: entity_id.0.clone(),
                    parent: parent_id,
                    components,
                });
            }
        }

        scene.entities.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(scene)
    }
}
