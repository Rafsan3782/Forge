//! Command pattern, undo/redo stack, and history for scene and tilemap edits.

use crate::ecs::World;
use crate::error::SceneError;
use crate::scene_ops::find_entity_by_id;
use crate::tilemap::Tilemap;
use forge_protocol::{
    ComponentFieldSchema, ComponentSchema, EditGetHistoryResult, FieldEditorHint, HistoryEntry,
};
use std::collections::HashMap;

/// Author of an edit command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandAuthor {
    /// Action executed by a human user in Studio.
    User,
    /// Action executed by an autonomous agent via bridge / MCP.
    Agent,
}

impl CommandAuthor {
    /// Returns the string representation ("user" or "agent").
    pub fn as_str(&self) -> &'static str {
        match self {
            CommandAuthor::User => "user",
            CommandAuthor::Agent => "agent",
        }
    }
}

/// An executable and undoable command representing a scene mutation.
#[derive(Debug, Clone)]
pub enum SceneCommand {
    /// Create entity.
    CreateEntity {
        /// Entity ID.
        entity_id: String,
        /// Name.
        name: String,
        /// Parent ID if any.
        parent_id: Option<String>,
        /// Components map.
        components: Option<HashMap<String, String>>,
    },
    /// Delete entity.
    DeleteEntity {
        /// Entity ID.
        entity_id: String,
        /// Full snapshot of entity and its descendants for restoration.
        serialized_subgraph: Vec<crate::scene::SceneEntity>,
    },
    /// Set component field or component data.
    SetComponent {
        /// Target entity ID.
        entity_id: String,
        /// Component name.
        component_name: String,
        /// New component field values.
        new_data: HashMap<String, String>,
        /// Previous component field values before application.
        prev_data: Option<HashMap<String, String>>,
    },
    /// Reparent entity.
    ReparentEntity {
        /// Target entity ID.
        entity_id: String,
        /// New parent ID if any.
        new_parent_id: Option<String>,
        /// Old parent ID before reparenting.
        old_parent_id: Option<String>,
    },
    /// Tilemap edit: set single tile.
    TilemapSetTile {
        /// Target entity ID.
        entity_id: String,
        /// Layer name.
        layer: String,
        /// Global tile coordinates.
        coord: [i32; 2],
        /// New tile ID.
        new_tile_id: i32,
        /// Previous tile ID before edit.
        prev_tile_id: i32,
    },
    /// Tilemap edit: fill rectangular region.
    TilemapFillRect {
        /// Target entity ID.
        entity_id: String,
        /// Layer name.
        layer: String,
        /// Min bounds.
        min: [i32; 2],
        /// Max bounds.
        max: [i32; 2],
        /// New tile ID.
        tile_id: i32,
        /// Saved previous tiles within the rectangle: (tx, ty) -> prev_tile_id.
        prev_tiles: Vec<([i32; 2], i32)>,
    },
    /// Tilemap edit: clear layer or all layers.
    TilemapClear {
        /// Target entity ID.
        entity_id: String,
        /// Layer name if specific layer was cleared.
        layer: Option<String>,
        /// Saved chunks before clearing.
        prev_chunks:
            std::collections::BTreeMap<crate::tilemap::ChunkCoord, crate::tilemap::TilemapChunk>,
    },
}

impl SceneCommand {
    /// Human-readable description of the command.
    pub fn description(&self) -> String {
        match self {
            SceneCommand::CreateEntity { name, .. } => format!("Create entity {}", name),
            SceneCommand::DeleteEntity { entity_id, .. } => format!("Delete entity {}", entity_id),
            SceneCommand::SetComponent {
                entity_id,
                component_name,
                ..
            } => format!("Set {} on {}", component_name, entity_id),
            SceneCommand::ReparentEntity {
                entity_id,
                new_parent_id,
                ..
            } => match new_parent_id {
                Some(p) => format!("Reparent {} to {}", entity_id, p),
                None => format!("Unparent {}", entity_id),
            },
            SceneCommand::TilemapSetTile {
                entity_id,
                coord,
                new_tile_id,
                ..
            } => format!(
                "Set tile on {} at [{}, {}] to {}",
                entity_id, coord[0], coord[1], new_tile_id
            ),
            SceneCommand::TilemapFillRect { entity_id, .. } => {
                format!("Fill rect on tilemap {}", entity_id)
            }
            SceneCommand::TilemapClear { entity_id, .. } => {
                format!("Clear tilemap {}", entity_id)
            }
        }
    }
}

/// A command item stored on the history stack.
#[derive(Debug, Clone)]
pub struct CommandEntry {
    /// Unique command ID.
    pub id: String,
    /// Author ("user" or "agent").
    pub author: CommandAuthor,
    /// Epoch timestamp in seconds.
    pub timestamp: f64,
    /// The command itself.
    pub command: SceneCommand,
}

/// Centralized undo/redo stack managing all scene modifications.
#[derive(Debug, Default, crate::ecs::Resource)]
pub struct CommandStack {
    /// Undo stack.
    undo_stack: Vec<CommandEntry>,
    /// Redo stack.
    redo_stack: Vec<CommandEntry>,
    /// Next command counter.
    counter: u64,
    /// Dirty tracking: number of commands since last save.
    clean_undo_len: usize,
    /// Current scene file path on disk.
    pub current_scene_path: Option<String>,
}

impl CommandStack {
    /// Creates a new empty command stack.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if there are unsaved scene modifications.
    pub fn is_dirty(&self) -> bool {
        self.undo_stack.len() != self.clean_undo_len
    }

    /// Marks the current state as clean (e.g. after scene.save).
    pub fn mark_clean(&mut self, scene_path: Option<String>) {
        self.clean_undo_len = self.undo_stack.len();
        if let Some(path) = scene_path {
            self.current_scene_path = Some(path);
        }
    }

    /// Pushes and executes a command on the world.
    pub fn execute(
        &mut self,
        world: &mut World,
        command: SceneCommand,
        author: CommandAuthor,
    ) -> Result<String, SceneError> {
        self.apply_do(world, &command)?;

        self.counter += 1;
        let id = format!("cmd_{}", self.counter);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        self.undo_stack.push(CommandEntry {
            id: id.clone(),
            author,
            timestamp,
            command,
        });

        // Any new action clears the redo stack
        self.redo_stack.clear();

        Ok(id)
    }

    /// Records an already-applied command onto the undo stack without re-executing it.
    pub fn record_executed(
        &mut self,
        command: SceneCommand,
        author: CommandAuthor,
    ) -> String {
        self.counter += 1;
        let id = format!("cmd_{}", self.counter);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        self.undo_stack.push(CommandEntry {
            id: id.clone(),
            author,
            timestamp,
            command,
        });

        self.redo_stack.clear();
        id
    }

    /// Undoes the last command, returning the undone command description.
    pub fn undo(&mut self, world: &mut World) -> Result<Option<String>, SceneError> {
        if let Some(entry) = self.undo_stack.pop() {
            self.apply_undo(world, &entry.command)?;
            let desc = entry.command.description();
            self.redo_stack.push(entry);
            Ok(Some(desc))
        } else {
            Ok(None)
        }
    }

    /// Redoes the next command on the redo stack.
    pub fn redo(&mut self, world: &mut World) -> Result<Option<String>, SceneError> {
        if let Some(entry) = self.redo_stack.pop() {
            self.apply_do(world, &entry.command)?;
            let desc = entry.command.description();
            self.undo_stack.push(entry);
            Ok(Some(desc))
        } else {
            Ok(None)
        }
    }

    /// Jumps to a specific command in the undo or redo history.
    pub fn jump_to(&mut self, world: &mut World, target_id: &str) -> Result<bool, SceneError> {
        // If target is in undo_stack, we undo until target is top of undo_stack
        if let Some(pos) = self.undo_stack.iter().position(|e| e.id == target_id) {
            while self.undo_stack.len() > pos + 1 {
                self.undo(world)?;
            }
            return Ok(true);
        }

        // If target is in redo_stack, we redo until target has been redone
        if let Some(pos) = self.redo_stack.iter().position(|e| e.id == target_id) {
            let redo_steps = self.redo_stack.len() - pos;
            for _ in 0..redo_steps {
                self.redo(world)?;
            }
            return Ok(true);
        }

        Ok(false)
    }

    /// Returns structured history of undo and redo stacks for Studio UI.
    pub fn get_history(&self) -> EditGetHistoryResult {
        let undo_entries = self
            .undo_stack
            .iter()
            .map(|e| HistoryEntry {
                id: e.id.clone(),
                description: e.command.description(),
                author: e.author.as_str().to_string(),
                timestamp: e.timestamp,
            })
            .collect();

        let redo_entries = self
            .redo_stack
            .iter()
            .map(|e| HistoryEntry {
                id: e.id.clone(),
                description: e.command.description(),
                author: e.author.as_str().to_string(),
                timestamp: e.timestamp,
            })
            .collect();

        EditGetHistoryResult {
            undo_stack: undo_entries,
            redo_stack: redo_entries,
            can_undo: !self.undo_stack.is_empty(),
            can_redo: !self.redo_stack.is_empty(),
        }
    }

    fn apply_do(&self, world: &mut World, command: &SceneCommand) -> Result<(), SceneError> {
        match command {
            SceneCommand::CreateEntity {
                entity_id,
                name,
                parent_id,
                components,
            } => {
                crate::scene_ops::raw_create_entity(
                    world,
                    entity_id,
                    name,
                    parent_id.as_deref(),
                    components.as_ref(),
                )?;
            }
            SceneCommand::DeleteEntity { entity_id, .. } => {
                crate::scene_ops::raw_delete_entity(world, entity_id)?;
            }
            SceneCommand::SetComponent {
                entity_id,
                component_name,
                new_data,
                ..
            } => {
                crate::scene_ops::raw_set_component(world, entity_id, component_name, new_data)?;
            }
            SceneCommand::ReparentEntity {
                entity_id,
                new_parent_id,
                ..
            } => {
                crate::scene_ops::raw_reparent_entity(world, entity_id, new_parent_id.as_deref())?;
            }
            SceneCommand::TilemapSetTile {
                entity_id,
                layer,
                coord,
                new_tile_id,
                ..
            } => {
                let entity = find_entity_by_id(world, entity_id)
                    .ok_or_else(|| SceneError::EntityNotFound(entity_id.clone()))?;
                let mut tm = world
                    .get_mut::<Tilemap>(entity)
                    .ok_or_else(|| SceneError::ComponentNotFound("Tilemap".to_string()))?;
                tm.set_tile(layer, coord[0], coord[1], *new_tile_id);
            }
            SceneCommand::TilemapFillRect {
                entity_id,
                layer,
                min,
                max,
                tile_id,
                ..
            } => {
                let entity = find_entity_by_id(world, entity_id)
                    .ok_or_else(|| SceneError::EntityNotFound(entity_id.clone()))?;
                let mut tm = world
                    .get_mut::<Tilemap>(entity)
                    .ok_or_else(|| SceneError::ComponentNotFound("Tilemap".to_string()))?;
                tm.fill_rect(layer, *min, *max, *tile_id);
            }
            SceneCommand::TilemapClear {
                entity_id, layer, ..
            } => {
                let entity = find_entity_by_id(world, entity_id)
                    .ok_or_else(|| SceneError::EntityNotFound(entity_id.clone()))?;
                let mut tm = world
                    .get_mut::<Tilemap>(entity)
                    .ok_or_else(|| SceneError::ComponentNotFound("Tilemap".to_string()))?;
                tm.clear_map(layer.as_deref());
            }
        }
        Ok(())
    }

    fn apply_undo(&self, world: &mut World, command: &SceneCommand) -> Result<(), SceneError> {
        match command {
            SceneCommand::CreateEntity { entity_id, .. } => {
                crate::scene_ops::raw_delete_entity(world, entity_id)?;
            }
            SceneCommand::DeleteEntity {
                serialized_subgraph,
                ..
            } => {
                // Restore all serialized entities from subgraph
                for se in serialized_subgraph {
                    crate::scene_ops::raw_restore_entity(world, se)?;
                }
            }
            SceneCommand::SetComponent {
                entity_id,
                component_name,
                prev_data,
                ..
            } => {
                if let Some(prev) = prev_data {
                    crate::scene_ops::raw_set_component(world, entity_id, component_name, prev)?;
                } else {
                    crate::scene_ops::raw_remove_component(world, entity_id, component_name)?;
                }
            }
            SceneCommand::ReparentEntity {
                entity_id,
                old_parent_id,
                ..
            } => {
                crate::scene_ops::raw_reparent_entity(world, entity_id, old_parent_id.as_deref())?;
            }
            SceneCommand::TilemapSetTile {
                entity_id,
                layer,
                coord,
                prev_tile_id,
                ..
            } => {
                let entity = find_entity_by_id(world, entity_id)
                    .ok_or_else(|| SceneError::EntityNotFound(entity_id.clone()))?;
                let mut tm = world
                    .get_mut::<Tilemap>(entity)
                    .ok_or_else(|| SceneError::ComponentNotFound("Tilemap".to_string()))?;
                tm.set_tile(layer, coord[0], coord[1], *prev_tile_id);
            }
            SceneCommand::TilemapFillRect {
                entity_id,
                layer,
                prev_tiles,
                ..
            } => {
                let entity = find_entity_by_id(world, entity_id)
                    .ok_or_else(|| SceneError::EntityNotFound(entity_id.clone()))?;
                let mut tm = world
                    .get_mut::<Tilemap>(entity)
                    .ok_or_else(|| SceneError::ComponentNotFound("Tilemap".to_string()))?;
                for (coord, tile_id) in prev_tiles {
                    tm.set_tile(layer, coord[0], coord[1], *tile_id);
                }
            }
            SceneCommand::TilemapClear {
                entity_id,
                prev_chunks,
                ..
            } => {
                let entity = find_entity_by_id(world, entity_id)
                    .ok_or_else(|| SceneError::EntityNotFound(entity_id.clone()))?;
                let mut tm = world
                    .get_mut::<Tilemap>(entity)
                    .ok_or_else(|| SceneError::ComponentNotFound("Tilemap".to_string()))?;
                tm.chunks = prev_chunks.clone();
            }
        }
        Ok(())
    }
}

/// Returns dynamic JSON schema metadata for all built-in components for Studio Inspector.
pub fn get_builtin_component_schemas() -> Vec<ComponentSchema> {
    vec![
        ComponentSchema {
            name: "Name".to_string(),
            display_name: Some("Name".to_string()),
            category: Some("Core".to_string()),
            fields: vec![ComponentFieldSchema {
                name: "name".to_string(),
                field_type: "text".to_string(),
                default_value: Some(serde_json::Value::String("Entity".to_string())),
                hint: Some(FieldEditorHint {
                    widget: "text".to_string(),
                    min: None,
                    max: None,
                    step: None,
                    asset_type: None,
                    options: None,
                }),
            }],
        },
        ComponentSchema {
            name: "Transform2D".to_string(),
            display_name: Some("Transform 2D".to_string()),
            category: Some("Spatial".to_string()),
            fields: vec![
                ComponentFieldSchema {
                    name: "position".to_string(),
                    field_type: "vector2".to_string(),
                    default_value: Some(serde_json::json!([0.0, 0.0])),
                    hint: Some(FieldEditorHint {
                        widget: "vector2".to_string(),
                        min: None,
                        max: None,
                        step: Some(0.5),
                        asset_type: None,
                        options: None,
                    }),
                },
                ComponentFieldSchema {
                    name: "rotation".to_string(),
                    field_type: "number".to_string(),
                    default_value: Some(serde_json::json!(0.0)),
                    hint: Some(FieldEditorHint {
                        widget: "number".to_string(),
                        min: Some(-360.0),
                        max: Some(360.0),
                        step: Some(1.0),
                        asset_type: None,
                        options: None,
                    }),
                },
                ComponentFieldSchema {
                    name: "scale".to_string(),
                    field_type: "vector2".to_string(),
                    default_value: Some(serde_json::json!([1.0, 1.0])),
                    hint: Some(FieldEditorHint {
                        widget: "vector2".to_string(),
                        min: Some(0.01),
                        max: Some(100.0),
                        step: Some(0.1),
                        asset_type: None,
                        options: None,
                    }),
                },
            ],
        },
        ComponentSchema {
            name: "Sprite".to_string(),
            display_name: Some("Sprite".to_string()),
            category: Some("2D Graphics".to_string()),
            fields: vec![
                ComponentFieldSchema {
                    name: "asset_id".to_string(),
                    field_type: "asset".to_string(),
                    default_value: Some(serde_json::Value::String("".to_string())),
                    hint: Some(FieldEditorHint {
                        widget: "asset".to_string(),
                        min: None,
                        max: None,
                        step: None,
                        asset_type: Some("texture".to_string()),
                        options: None,
                    }),
                },
                ComponentFieldSchema {
                    name: "size".to_string(),
                    field_type: "vector2".to_string(),
                    default_value: Some(serde_json::json!([32.0, 32.0])),
                    hint: Some(FieldEditorHint {
                        widget: "vector2".to_string(),
                        min: Some(1.0),
                        max: Some(2048.0),
                        step: Some(1.0),
                        asset_type: None,
                        options: None,
                    }),
                },
                ComponentFieldSchema {
                    name: "color_tint".to_string(),
                    field_type: "color".to_string(),
                    default_value: Some(serde_json::json!([1.0, 1.0, 1.0, 1.0])),
                    hint: Some(FieldEditorHint {
                        widget: "color".to_string(),
                        min: None,
                        max: None,
                        step: None,
                        asset_type: None,
                        options: None,
                    }),
                },
                ComponentFieldSchema {
                    name: "z_order".to_string(),
                    field_type: "number".to_string(),
                    default_value: Some(serde_json::json!(0)),
                    hint: Some(FieldEditorHint {
                        widget: "number".to_string(),
                        min: Some(-100.0),
                        max: Some(100.0),
                        step: Some(1.0),
                        asset_type: None,
                        options: None,
                    }),
                },
            ],
        },
        ComponentSchema {
            name: "Camera2D".to_string(),
            display_name: Some("Camera 2D".to_string()),
            category: Some("Graphics".to_string()),
            fields: vec![
                ComponentFieldSchema {
                    name: "zoom".to_string(),
                    field_type: "number".to_string(),
                    default_value: Some(serde_json::json!(1.0)),
                    hint: Some(FieldEditorHint {
                        widget: "number".to_string(),
                        min: Some(0.1),
                        max: Some(10.0),
                        step: Some(0.1),
                        asset_type: None,
                        options: None,
                    }),
                },
                ComponentFieldSchema {
                    name: "is_active".to_string(),
                    field_type: "bool".to_string(),
                    default_value: Some(serde_json::json!(true)),
                    hint: Some(FieldEditorHint {
                        widget: "bool".to_string(),
                        min: None,
                        max: None,
                        step: None,
                        asset_type: None,
                        options: None,
                    }),
                },
            ],
        },
        ComponentSchema {
            name: "RigidBody2D".to_string(),
            display_name: Some("Rigid Body 2D".to_string()),
            category: Some("Physics".to_string()),
            fields: vec![
                ComponentFieldSchema {
                    name: "body_type".to_string(),
                    field_type: "enum".to_string(),
                    default_value: Some(serde_json::Value::String("dynamic".to_string())),
                    hint: Some(FieldEditorHint {
                        widget: "enum".to_string(),
                        min: None,
                        max: None,
                        step: None,
                        asset_type: None,
                        options: Some(vec![
                            "dynamic".to_string(),
                            "fixed".to_string(),
                            "kinematic".to_string(),
                        ]),
                    }),
                },
                ComponentFieldSchema {
                    name: "gravity_scale".to_string(),
                    field_type: "number".to_string(),
                    default_value: Some(serde_json::json!(1.0)),
                    hint: Some(FieldEditorHint {
                        widget: "number".to_string(),
                        min: Some(-10.0),
                        max: Some(10.0),
                        step: Some(0.1),
                        asset_type: None,
                        options: None,
                    }),
                },
                ComponentFieldSchema {
                    name: "linear_damping".to_string(),
                    field_type: "number".to_string(),
                    default_value: Some(serde_json::json!(0.0)),
                    hint: Some(FieldEditorHint {
                        widget: "number".to_string(),
                        min: Some(0.0),
                        max: Some(10.0),
                        step: Some(0.1),
                        asset_type: None,
                        options: None,
                    }),
                },
            ],
        },
        ComponentSchema {
            name: "Collider2D".to_string(),
            display_name: Some("Collider 2D".to_string()),
            category: Some("Physics".to_string()),
            fields: vec![
                ComponentFieldSchema {
                    name: "shape".to_string(),
                    field_type: "enum".to_string(),
                    default_value: Some(serde_json::Value::String("box".to_string())),
                    hint: Some(FieldEditorHint {
                        widget: "enum".to_string(),
                        min: None,
                        max: None,
                        step: None,
                        asset_type: None,
                        options: Some(vec!["box".to_string(), "circle".to_string()]),
                    }),
                },
                ComponentFieldSchema {
                    name: "size".to_string(),
                    field_type: "vector2".to_string(),
                    default_value: Some(serde_json::json!([32.0, 32.0])),
                    hint: Some(FieldEditorHint {
                        widget: "vector2".to_string(),
                        min: Some(1.0),
                        max: Some(1000.0),
                        step: Some(1.0),
                        asset_type: None,
                        options: None,
                    }),
                },
                ComponentFieldSchema {
                    name: "is_sensor".to_string(),
                    field_type: "bool".to_string(),
                    default_value: Some(serde_json::json!(false)),
                    hint: Some(FieldEditorHint {
                        widget: "bool".to_string(),
                        min: None,
                        max: None,
                        step: None,
                        asset_type: None,
                        options: None,
                    }),
                },
                ComponentFieldSchema {
                    name: "friction".to_string(),
                    field_type: "number".to_string(),
                    default_value: Some(serde_json::json!(0.5)),
                    hint: Some(FieldEditorHint {
                        widget: "number".to_string(),
                        min: Some(0.0),
                        max: Some(2.0),
                        step: Some(0.05),
                        asset_type: None,
                        options: None,
                    }),
                },
            ],
        },
        ComponentSchema {
            name: "Script".to_string(),
            display_name: Some("Script".to_string()),
            category: Some("Logic".to_string()),
            fields: vec![
                ComponentFieldSchema {
                    name: "path".to_string(),
                    field_type: "asset".to_string(),
                    default_value: Some(serde_json::Value::String("".to_string())),
                    hint: Some(FieldEditorHint {
                        widget: "asset".to_string(),
                        min: None,
                        max: None,
                        step: None,
                        asset_type: Some("script".to_string()),
                        options: None,
                    }),
                },
                ComponentFieldSchema {
                    name: "enabled".to_string(),
                    field_type: "bool".to_string(),
                    default_value: Some(serde_json::json!(true)),
                    hint: Some(FieldEditorHint {
                        widget: "bool".to_string(),
                        min: None,
                        max: None,
                        step: None,
                        asset_type: None,
                        options: None,
                    }),
                },
            ],
        },
        ComponentSchema {
            name: "Tilemap".to_string(),
            display_name: Some("Tilemap".to_string()),
            category: Some("2D Graphics".to_string()),
            fields: vec![
                ComponentFieldSchema {
                    name: "tileset_id".to_string(),
                    field_type: "asset".to_string(),
                    default_value: Some(serde_json::Value::String("".to_string())),
                    hint: Some(FieldEditorHint {
                        widget: "asset".to_string(),
                        min: None,
                        max: None,
                        step: None,
                        asset_type: Some("tileset".to_string()),
                        options: None,
                    }),
                },
                ComponentFieldSchema {
                    name: "tile_size".to_string(),
                    field_type: "vector2".to_string(),
                    default_value: Some(serde_json::json!([16.0, 16.0])),
                    hint: Some(FieldEditorHint {
                        widget: "vector2".to_string(),
                        min: Some(4.0),
                        max: Some(256.0),
                        step: Some(1.0),
                        asset_type: None,
                        options: None,
                    }),
                },
                ComponentFieldSchema {
                    name: "chunk_size".to_string(),
                    field_type: "number".to_string(),
                    default_value: Some(serde_json::json!(16)),
                    hint: Some(FieldEditorHint {
                        widget: "number".to_string(),
                        min: Some(8.0),
                        max: Some(64.0),
                        step: Some(8.0),
                        asset_type: None,
                        options: None,
                    }),
                },
            ],
        },
    ]
}
