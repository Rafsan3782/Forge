//! Model Context Protocol (MCP) tool schemas and message types.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// MCP Tool descriptor schema matching the MCP specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// Unique tool name (e.g. "scene.get_tree").
    pub name: String,
    /// Human and LLM readable tool description.
    pub description: String,
    /// JSON Schema defining valid input arguments.
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// MCP `tools/list` response payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolsListResult {
    /// List of available tools exposed by the engine bridge.
    pub tools: Vec<McpTool>,
}

/// Parameters passed to `tools/call`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCallParams {
    /// Name of the tool to invoke.
    pub name: String,
    /// Arguments payload matching the tool's input schema.
    #[serde(default)]
    pub arguments: Value,
}

/// MCP content block in a tool call result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContentBlock {
    /// Content type (typically "text").
    #[serde(rename = "type")]
    pub content_type: String,
    /// Textual payload.
    pub text: String,
}

/// MCP `tools/call` result envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCallResult {
    /// Result content blocks.
    pub content: Vec<McpContentBlock>,
    /// Whether tool execution resulted in an error.
    #[serde(rename = "isError")]
    pub is_error: bool,
}

impl McpToolCallResult {
    /// Creates a successful MCP result from a JSON value.
    pub fn success<T: Serialize>(value: &T) -> Result<Self, serde_json::Error> {
        let text = serde_json::to_string(value)?;
        Ok(Self {
            content: vec![McpContentBlock {
                content_type: "text".to_string(),
                text,
            }],
            is_error: false,
        })
    }

    /// Creates an error MCP result from an error message string.
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            content: vec![McpContentBlock {
                content_type: "text".to_string(),
                text: msg.into(),
            }],
            is_error: true,
        }
    }
}

/// Returns the complete registry of MCP tools exposed by Forge Bridge.
pub fn get_mcp_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "engine.ping".to_string(),
            description: "Heartbeat and liveness check for the Forge engine".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "timestamp": {
                        "type": "number",
                        "description": "Timestamp in milliseconds"
                    }
                }
            }),
        },
        McpTool {
            name: "scene.get_tree".to_string(),
            description: "Retrieve the current entity hierarchy tree from the active scene".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "root_entity_id": {
                        "type": ["string", "null"],
                        "description": "Optional root entity ID to restrict subtree query"
                    }
                }
            }),
        },
        McpTool {
            name: "scene.create_entity".to_string(),
            description: "Create a new entity with optional parent and initial components".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["name"],
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Display name of the new entity"
                    },
                    "parent_id": {
                        "type": ["string", "null"],
                        "description": "Optional parent entity ID"
                    },
                    "components": {
                        "type": ["object", "null"],
                        "description": "Map of component names to JSON string representations"
                    }
                }
            }),
        },
        McpTool {
            name: "scene.set_component".to_string(),
            description: "Set or update a component on an entity".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["entity_id", "component_name", "data"],
                "properties": {
                    "entity_id": {
                        "type": "string",
                        "description": "Target entity ID"
                    },
                    "component_name": {
                        "type": "string",
                        "description": "Component type name (e.g. Transform2D, Sprite, Name)"
                    },
                    "data": {
                        "type": "object",
                        "description": "Component property key-value map"
                    }
                }
            }),
        },
        McpTool {
            name: "scene.delete_entity".to_string(),
            description: "Delete an entity and all its descendants from the world".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["entity_id"],
                "properties": {
                    "entity_id": {
                        "type": "string",
                        "description": "Target entity ID to delete"
                    }
                }
            }),
        },
        McpTool {
            name: "game.screenshot".to_string(),
            description: "Capture a viewport screenshot from the running game instance as base64 or saved file path".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "format": {
                        "type": "string",
                        "enum": ["png", "jpeg"],
                        "default": "png"
                    },
                    "width": {
                        "type": ["integer", "null"],
                        "description": "Optional custom capture width"
                    },
                    "height": {
                        "type": ["integer", "null"],
                        "description": "Optional custom capture height"
                    },
                    "file_path": {
                        "type": ["string", "null"],
                        "description": "Optional file path to write the PNG directly on disk"
                    }
                }
            }),
        },
        McpTool {
            name: "game.simulate_input".to_string(),
            description: "Simulate input events (key press/release, action, axis, mouse) on the engine thread".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["events"],
                "properties": {
                    "events": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "required": ["input_type"],
                            "properties": {
                                "input_type": {
                                    "type": "string",
                                    "enum": ["key_down", "key_up", "action_down", "action_up", "action_axis", "mouse_move", "mouse_down", "mouse_up"]
                                },
                                "key": { "type": "string" },
                                "action": { "type": "string" },
                                "value": { "type": "number" },
                                "duration_frames": { "type": "integer", "default": 1 }
                            }
                        }
                    }
                }
            }),
        },
        McpTool {
            name: "project.stop".to_string(),
            description: "Stop and terminate the running Forge game runtime session".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "session_id": {
                        "type": "string",
                        "description": "Session ID or empty string"
                    }
                }
            }),
        },
        McpTool {
            name: "asset.list".to_string(),
            description: "List all tracked assets in the project asset database, optionally filtered by asset type".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "asset_type": {
                        "type": ["string", "null"],
                        "description": "Optional asset type filter (e.g. texture, spritesheet, animation, scene)"
                    }
                }
            }),
        },
        McpTool {
            name: "asset.get_info".to_string(),
            description: "Retrieve metadata, path, and type information for an asset by its unique UUID".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["id"],
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Stable UUID of the asset"
                    }
                }
            }),
        },
        McpTool {
            name: "asset.import".to_string(),
            description: "Copy an external file into the project assets folder and generate a stable sidecar metadata file".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["source_path", "target_path"],
                "properties": {
                    "source_path": {
                        "type": "string",
                        "description": "Path to the source file to import"
                    },
                    "target_path": {
                        "type": "string",
                        "description": "Relative target path inside the project assets directory"
                    },
                    "asset_type": {
                        "type": ["string", "null"],
                        "description": "Optional explicit asset type"
                    },
                    "metadata": {
                        "type": ["object", "null"],
                        "description": "Optional initial metadata or import settings"
                    }
                }
            }),
        },
        McpTool {
            name: "physics.raycast".to_string(),
            description: "Cast a ray through the physics simulation world and retrieve the first intersecting collider".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["origin", "direction"],
                "properties": {
                    "origin": {
                        "type": "array",
                        "items": { "type": "number" },
                        "minItems": 2,
                        "maxItems": 2,
                        "description": "Ray origin in world coordinates [x, y]"
                    },
                    "direction": {
                        "type": "array",
                        "items": { "type": "number" },
                        "minItems": 2,
                        "maxItems": 2,
                        "description": "Ray direction vector [dx, dy]"
                    },
                    "max_distance": {
                        "type": ["number", "null"],
                        "description": "Optional maximum ray length (default 1000.0)"
                    },
                    "collision_mask": {
                        "type": ["integer", "null"],
                        "description": "Optional bitmask to filter collider layers"
                    },
                    "query_sensors": {
                        "type": ["boolean", "null"],
                        "description": "Whether to hit trigger/sensor colliders (default false)"
                    }
                }
            }),
        },
        McpTool {
            name: "tilemap.get".to_string(),
            description: "Retrieve the tile ID at specific tile coordinates on a tilemap layer".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["entity_id", "x", "y"],
                "properties": {
                    "entity_id": {
                        "type": "string",
                        "description": "ID of the entity containing the Tilemap component"
                    },
                    "layer": {
                        "type": ["string", "null"],
                        "description": "Layer name (defaults to main)"
                    },
                    "x": {
                        "type": "integer",
                        "description": "Tile X coordinate"
                    },
                    "y": {
                        "type": "integer",
                        "description": "Tile Y coordinate"
                    }
                }
            }),
        },
        McpTool {
            name: "tilemap.set_tile".to_string(),
            description: "Set a tile ID at specific tile coordinates on a tilemap layer".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["entity_id", "x", "y", "tile_id"],
                "properties": {
                    "entity_id": {
                        "type": "string",
                        "description": "ID of the entity containing the Tilemap component"
                    },
                    "layer": {
                        "type": ["string", "null"],
                        "description": "Layer name (defaults to main)"
                    },
                    "x": {
                        "type": "integer",
                        "description": "Tile X coordinate"
                    },
                    "y": {
                        "type": "integer",
                        "description": "Tile Y coordinate"
                    },
                    "tile_id": {
                        "type": "integer",
                        "description": "Tile ID to set (0 to clear)"
                    }
                }
            }),
        },
        McpTool {
            name: "tilemap.fill_rect".to_string(),
            description: "Fill a rectangle of tiles on a tilemap layer".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["entity_id", "min", "max", "tile_id"],
                "properties": {
                    "entity_id": {
                        "type": "string",
                        "description": "ID of the entity containing the Tilemap component"
                    },
                    "layer": {
                        "type": ["string", "null"],
                        "description": "Layer name (defaults to main)"
                    },
                    "min": {
                        "type": "array",
                        "items": { "type": "integer" },
                        "minItems": 2,
                        "maxItems": 2,
                        "description": "Minimum tile coordinates [min_x, min_y]"
                    },
                    "max": {
                        "type": "array",
                        "items": { "type": "integer" },
                        "minItems": 2,
                        "maxItems": 2,
                        "description": "Maximum tile coordinates [max_x, max_y]"
                    },
                    "tile_id": {
                        "type": "integer",
                        "description": "Tile ID to fill"
                    }
                }
            }),
        },
        McpTool {
            name: "tilemap.clear".to_string(),
            description: "Clear all tiles or an entire layer from a tilemap".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["entity_id"],
                "properties": {
                    "entity_id": {
                        "type": "string",
                        "description": "ID of the entity containing the Tilemap component"
                    },
                    "layer": {
                        "type": ["string", "null"],
                        "description": "Optional specific layer to clear"
                    }
                }
            }),
        },
        McpTool {
            name: "audio.play".to_string(),
            description: "Play an audio sound or music track by asset ID".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["asset_id"],
                "properties": {
                    "asset_id": {
                        "type": "string",
                        "description": "Asset ID or path of audio file to play"
                    },
                    "bus": {
                        "type": "string",
                        "enum": ["master", "music", "sfx"],
                        "description": "Bus to route audio to (default sfx)"
                    },
                    "volume": {
                        "type": "number",
                        "description": "Volume multiplier (default 1.0)"
                    },
                    "pitch": {
                        "type": "number",
                        "description": "Pitch multiplier (default 1.0)"
                    },
                    "loop": {
                        "type": "boolean",
                        "description": "Whether playback loops indefinitely"
                    },
                    "fade_in_seconds": {
                        "type": "number",
                        "description": "Optional fade in duration in seconds"
                    }
                }
            }),
        },
        McpTool {
            name: "audio.stop".to_string(),
            description: "Stop audio playback for a sound ID or bus".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "sound_id": {
                        "type": ["string", "null"],
                        "description": "Optional playback instance ID"
                    },
                    "bus": {
                        "type": ["string", "null"],
                        "enum": ["master", "music", "sfx"],
                        "description": "Optional bus to stop all playback on"
                    },
                    "fade_out_seconds": {
                        "type": "number",
                        "description": "Optional fade out duration in seconds"
                    }
                }
            }),
        },
        McpTool {
            name: "audio.set_bus_volume".to_string(),
            description: "Set the volume of an audio bus".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["bus", "volume"],
                "properties": {
                    "bus": {
                        "type": "string",
                        "enum": ["master", "music", "sfx"],
                        "description": "Audio bus channel"
                    },
                    "volume": {
                        "type": "number",
                        "description": "Volume multiplier from 0.0 to 1.0"
                    }
                }
            }),
        },
        McpTool {
            name: "script.check".to_string(),
            description: "Run TypeScript type checking and return structured diagnostics".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": ["string", "null"],
                        "description": "Optional project root directory containing scripts/ and tsconfig.json"
                    }
                }
            }),
        },
        McpTool {
            name: "script.list".to_string(),
            description: "List all TypeScript script files in a project scripts directory".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": ["string", "null"],
                        "description": "Optional project root directory"
                    }
                }
            }),
        },
        McpTool {
            name: "script.read".to_string(),
            description: "Read the source text of a script file".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["path"],
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path to the script"
                    }
                }
            }),
        },
        McpTool {
            name: "script.write".to_string(),
            description: "Validate TypeScript syntax with SWC and write script file content".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["path", "content"],
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path where script should be written"
                    },
                    "content": {
                        "type": "string",
                        "description": "TypeScript script source code"
                    }
                }
            }),
        },
        McpTool {
            name: "play.start".to_string(),
            description: "Snapshot the active scene and start play mode simulation (scripts, physics, audio)".to_string(),
            input_schema: json!({ "type": "object", "properties": {} }),
        },
        McpTool {
            name: "play.pause".to_string(),
            description: "Pause play mode simulation".to_string(),
            input_schema: json!({ "type": "object", "properties": {} }),
        },
        McpTool {
            name: "play.step".to_string(),
            description: "Advance simulation by N frames while paused".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "frames": { "type": "integer", "default": 1 }
                }
            }),
        },
        McpTool {
            name: "play.stop".to_string(),
            description: "Stop simulation and restore exact scene snapshot from before play started".to_string(),
            input_schema: json!({ "type": "object", "properties": {} }),
        },
    ]
}
