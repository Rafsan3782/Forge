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
    ]
}
