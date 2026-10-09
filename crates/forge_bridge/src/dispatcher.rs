//! Request dispatcher handling JSON-RPC 2.0 and MCP tool requests.

use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::oneshot;

use forge_protocol::{
    EnginePingParams, GameScreenshotParams, ProjectStopParams, SceneCreateEntityParams,
    SceneDeleteEntityParams, SceneGetTreeParams, SceneSetComponentParams,
};

use crate::command::{BridgeCommand, BridgeCommandSender};
use crate::error::BridgeError;
use crate::mcp::{get_mcp_tools, McpToolCallParams, McpToolCallResult, McpToolsListResult};

/// Dispatches an incoming JSON-RPC method call to the engine or internal handlers.
pub async fn dispatch_method(
    method: &str,
    params: Value,
    command_sender: &BridgeCommandSender,
) -> Result<Value, BridgeError> {
    match method {
        // ---------------------------------------------------------------------
        // MCP Specification Endpoints
        // ---------------------------------------------------------------------
        "tools/list" => {
            let result = McpToolsListResult {
                tools: get_mcp_tools(),
            };
            Ok(serde_json::to_value(result)?)
        }
        "tools/call" => {
            let call_params: McpToolCallParams = serde_json::from_value(params).map_err(|e| {
                BridgeError::InvalidParams(format!("Failed to parse tools/call params: {}", e))
            })?;

            match dispatch_direct_tool(&call_params.name, call_params.arguments, command_sender)
                .await
            {
                Ok(val) => {
                    let mcp_res = McpToolCallResult::success(&val).map_err(BridgeError::Json)?;
                    Ok(serde_json::to_value(mcp_res)?)
                }
                Err(e) => {
                    let mcp_res = McpToolCallResult::error(e.to_string());
                    Ok(serde_json::to_value(mcp_res)?)
                }
            }
        }

        // ---------------------------------------------------------------------
        // Direct JSON-RPC Tool Endpoints
        // ---------------------------------------------------------------------
        direct => dispatch_direct_tool(direct, params, command_sender).await,
    }
}

/// Dispatches an individual tool execution against the engine thread.
pub async fn dispatch_direct_tool(
    tool_name: &str,
    params: Value,
    command_sender: &BridgeCommandSender,
) -> Result<Value, BridgeError> {
    match tool_name {
        "engine.ping" => {
            let ping_params: EnginePingParams = if params.is_null() {
                EnginePingParams::default()
            } else {
                serde_json::from_value(params).unwrap_or_default()
            };

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::EnginePing {
                    params: ping_params.clone(),
                    responder: tx,
                })
                .map_err(|_| BridgeError::ChannelClosed)?;

            let result = rx
                .await
                .map_err(|_| BridgeError::ChannelClosed)?
                .map_err(BridgeError::Engine)?;

            Ok(serde_json::to_value(result)?)
        }

        "scene.get_tree" => {
            let tree_params: SceneGetTreeParams = if params.is_null() {
                SceneGetTreeParams::default()
            } else {
                serde_json::from_value(params)
                    .map_err(|e| BridgeError::InvalidParams(e.to_string()))?
            };

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::SceneGetTree {
                    params: tree_params,
                    responder: tx,
                })
                .map_err(|_| BridgeError::ChannelClosed)?;

            let result = rx
                .await
                .map_err(|_| BridgeError::ChannelClosed)?
                .map_err(BridgeError::Engine)?;

            Ok(serde_json::to_value(result)?)
        }

        "scene.create_entity" => {
            let create_params: SceneCreateEntityParams = serde_json::from_value(params)
                .map_err(|e| BridgeError::InvalidParams(e.to_string()))?;

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::SceneCreateEntity {
                    params: create_params,
                    responder: tx,
                })
                .map_err(|_| BridgeError::ChannelClosed)?;

            let result = rx
                .await
                .map_err(|_| BridgeError::ChannelClosed)?
                .map_err(BridgeError::Engine)?;

            Ok(serde_json::to_value(result)?)
        }

        "scene.set_component" => {
            let set_params: SceneSetComponentParams = serde_json::from_value(params)
                .map_err(|e| BridgeError::InvalidParams(e.to_string()))?;

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::SceneSetComponent {
                    params: set_params,
                    responder: tx,
                })
                .map_err(|_| BridgeError::ChannelClosed)?;

            let result = rx
                .await
                .map_err(|_| BridgeError::ChannelClosed)?
                .map_err(BridgeError::Engine)?;

            Ok(serde_json::to_value(result)?)
        }

        "scene.delete_entity" => {
            let del_params: SceneDeleteEntityParams = serde_json::from_value(params)
                .map_err(|e| BridgeError::InvalidParams(e.to_string()))?;

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::SceneDeleteEntity {
                    params: del_params,
                    responder: tx,
                })
                .map_err(|_| BridgeError::ChannelClosed)?;

            let result = rx
                .await
                .map_err(|_| BridgeError::ChannelClosed)?
                .map_err(BridgeError::Engine)?;

            Ok(serde_json::to_value(result)?)
        }

        "game.screenshot" => {
            let shot_params: GameScreenshotParams = if params.is_null() {
                serde_json::from_value(json!({})).unwrap()
            } else {
                serde_json::from_value(params)
                    .map_err(|e| BridgeError::InvalidParams(e.to_string()))?
            };

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::GameScreenshot {
                    params: shot_params,
                    responder: tx,
                })
                .map_err(|_| BridgeError::ChannelClosed)?;

            let result = rx
                .await
                .map_err(|_| BridgeError::ChannelClosed)?
                .map_err(BridgeError::Engine)?;

            Ok(serde_json::to_value(result)?)
        }

        "game.simulate_input" => {
            let sim_params: forge_protocol::GameSimulateInputParams =
                serde_json::from_value(params)
                    .map_err(|e| BridgeError::InvalidParams(e.to_string()))?;

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::GameSimulateInput {
                    params: sim_params,
                    responder: tx,
                })
                .map_err(|_| BridgeError::ChannelClosed)?;

            let result = rx
                .await
                .map_err(|_| BridgeError::ChannelClosed)?
                .map_err(BridgeError::Engine)?;

            Ok(serde_json::to_value(result)?)
        }

        "project.stop" => {
            let stop_params: ProjectStopParams = if params.is_null() {
                ProjectStopParams::default()
            } else {
                serde_json::from_value(params).unwrap_or_default()
            };

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::ProjectStop {
                    params: stop_params,
                    responder: tx,
                })
                .map_err(|_| BridgeError::ChannelClosed)?;

            let result = rx
                .await
                .map_err(|_| BridgeError::ChannelClosed)?
                .map_err(BridgeError::Engine)?;

            Ok(serde_json::to_value(result)?)
        }

        unknown => Err(BridgeError::MethodNotFound(unknown.to_string())),
    }
}

/// Helper function to retrieve current timestamp in milliseconds.
pub fn current_timestamp_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
}
