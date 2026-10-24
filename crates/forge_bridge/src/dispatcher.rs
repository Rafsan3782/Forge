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

        "asset.list" => {
            let list_params: forge_protocol::AssetListParams = if params.is_null() {
                forge_protocol::AssetListParams::default()
            } else {
                serde_json::from_value(params)
                    .map_err(|e| BridgeError::InvalidParams(e.to_string()))?
            };

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::AssetList {
                    params: list_params,
                    responder: tx,
                })
                .map_err(|_| BridgeError::ChannelClosed)?;

            let result = rx
                .await
                .map_err(|_| BridgeError::ChannelClosed)?
                .map_err(BridgeError::Engine)?;

            Ok(serde_json::to_value(result)?)
        }

        "asset.get_info" => {
            let info_params: forge_protocol::AssetGetInfoParams = serde_json::from_value(params)
                .map_err(|e| BridgeError::InvalidParams(e.to_string()))?;

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::AssetGetInfo {
                    params: info_params,
                    responder: tx,
                })
                .map_err(|_| BridgeError::ChannelClosed)?;

            let result = rx
                .await
                .map_err(|_| BridgeError::ChannelClosed)?
                .map_err(BridgeError::Engine)?;

            Ok(serde_json::to_value(result)?)
        }

        "asset.import" => {
            let import_params: forge_protocol::AssetImportParams =
                serde_json::from_value(params)
                    .map_err(|e| BridgeError::InvalidParams(e.to_string()))?;

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::AssetImport {
                    params: import_params,
                    responder: tx,
                })
                .map_err(|_| BridgeError::ChannelClosed)?;

            let result = rx
                .await
                .map_err(|_| BridgeError::ChannelClosed)?
                .map_err(BridgeError::Engine)?;

            Ok(serde_json::to_value(result)?)
        }

        "physics.raycast" => {
            let ray_params: forge_protocol::PhysicsRaycastParams =
                serde_json::from_value(params)
                    .map_err(|e| BridgeError::InvalidParams(e.to_string()))?;

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::PhysicsRaycast {
                    params: ray_params,
                    responder: tx,
                })
                .map_err(|_| BridgeError::ChannelClosed)?;

            let result = rx
                .await
                .map_err(|_| BridgeError::ChannelClosed)?
                .map_err(BridgeError::Engine)?;

            Ok(serde_json::to_value(result)?)
        }

        "tilemap.get" => {
            let tile_params: forge_protocol::TilemapGetParams = serde_json::from_value(params)
                .map_err(|e| BridgeError::InvalidParams(e.to_string()))?;

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::TilemapGet {
                    params: tile_params,
                    responder: tx,
                })
                .map_err(|_| BridgeError::ChannelClosed)?;

            let result = rx
                .await
                .map_err(|_| BridgeError::ChannelClosed)?
                .map_err(BridgeError::Engine)?;

            Ok(serde_json::to_value(result)?)
        }

        "tilemap.set_tile" => {
            let tile_params: forge_protocol::TilemapSetTileParams = serde_json::from_value(params)
                .map_err(|e| BridgeError::InvalidParams(e.to_string()))?;

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::TilemapSetTile {
                    params: tile_params,
                    responder: tx,
                })
                .map_err(|_| BridgeError::ChannelClosed)?;

            let result = rx
                .await
                .map_err(|_| BridgeError::ChannelClosed)?
                .map_err(BridgeError::Engine)?;

            Ok(serde_json::to_value(result)?)
        }

        "tilemap.fill_rect" => {
            let fill_params: forge_protocol::TilemapFillRectParams = serde_json::from_value(params)
                .map_err(|e| BridgeError::InvalidParams(e.to_string()))?;

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::TilemapFillRect {
                    params: fill_params,
                    responder: tx,
                })
                .map_err(|_| BridgeError::ChannelClosed)?;

            let result = rx
                .await
                .map_err(|_| BridgeError::ChannelClosed)?
                .map_err(BridgeError::Engine)?;

            Ok(serde_json::to_value(result)?)
        }

        "tilemap.clear" => {
            let clear_params: forge_protocol::TilemapClearParams =
                serde_json::from_value(params)
                    .map_err(|e| BridgeError::InvalidParams(e.to_string()))?;

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::TilemapClear {
                    params: clear_params,
                    responder: tx,
                })
                .map_err(|_| BridgeError::ChannelClosed)?;

            let result = rx
                .await
                .map_err(|_| BridgeError::ChannelClosed)?
                .map_err(BridgeError::Engine)?;

            Ok(serde_json::to_value(result)?)
        }

        "audio.play" => {
            let play_params: forge_protocol::AudioPlayParams = serde_json::from_value(params)
                .map_err(|e| BridgeError::InvalidParams(e.to_string()))?;

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::AudioPlay {
                    params: play_params,
                    responder: tx,
                })
                .map_err(|_| BridgeError::ChannelClosed)?;

            let result = rx
                .await
                .map_err(|_| BridgeError::ChannelClosed)?
                .map_err(BridgeError::Engine)?;

            Ok(serde_json::to_value(result)?)
        }

        "audio.stop" => {
            let stop_params: forge_protocol::AudioStopParams = if params.is_null() {
                forge_protocol::AudioStopParams::default()
            } else {
                serde_json::from_value(params)
                    .map_err(|e| BridgeError::InvalidParams(e.to_string()))?
            };

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::AudioStop {
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

        "audio.set_bus_volume" => {
            let vol_params: forge_protocol::AudioSetBusVolumeParams =
                serde_json::from_value(params)
                    .map_err(|e| BridgeError::InvalidParams(e.to_string()))?;

            let (tx, rx) = oneshot::channel();
            command_sender
                .send(BridgeCommand::AudioSetBusVolume {
                    params: vol_params,
                    responder: tx,
                })
                .map_err(|_| BridgeError::ChannelClosed)?;

            let result = rx
                .await
                .map_err(|_| BridgeError::ChannelClosed)?
                .map_err(BridgeError::Engine)?;

            Ok(serde_json::to_value(result)?)
        }

        "script.check" => {
            let check_params: forge_protocol::ScriptCheckParams = if params.is_null() {
                serde_json::from_value(json!({})).unwrap()
            } else {
                serde_json::from_value(params)
                    .map_err(|e| BridgeError::InvalidParams(e.to_string()))?
            };

            let (success, diagnostics) =
                forge_cli::run_typecheck(check_params.project_path.as_deref())
                    .map_err(BridgeError::Engine)?;

            let result = forge_protocol::ScriptCheckResult {
                success,
                diagnostics,
            };

            Ok(serde_json::to_value(result)?)
        }

        "script.list" => {
            let list_params: forge_protocol::ScriptListParams = if params.is_null() {
                serde_json::from_value(json!({})).unwrap()
            } else {
                serde_json::from_value(params)
                    .map_err(|e| BridgeError::InvalidParams(e.to_string()))?
            };

            let root = list_params.project_path.unwrap_or_else(|| ".".to_string());
            let scripts_dir = std::path::Path::new(&root).join("scripts");
            let mut scripts = Vec::new();

            if scripts_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&scripts_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) == Some("ts") {
                            scripts.push(path.to_string_lossy().replace('\\', "/"));
                        }
                    }
                }
            }

            let result = forge_protocol::ScriptListResult { scripts };
            Ok(serde_json::to_value(result)?)
        }

        "script.read" => {
            let read_params: forge_protocol::ScriptReadParams = serde_json::from_value(params)
                .map_err(|e| BridgeError::InvalidParams(e.to_string()))?;

            let content = std::fs::read_to_string(&read_params.path).map_err(|e| {
                BridgeError::Engine(format!("Failed to read script {}: {}", read_params.path, e))
            })?;

            let result = forge_protocol::ScriptReadResult {
                path: read_params.path,
                content,
            };
            Ok(serde_json::to_value(result)?)
        }

        "script.write" => {
            let write_params: forge_protocol::ScriptWriteParams = serde_json::from_value(params)
                .map_err(|e| BridgeError::InvalidParams(e.to_string()))?;

            // Validate syntax with SWC before saving
            match forge_script::transpile_typescript(&write_params.path, &write_params.content) {
                Ok(_) => {
                    if let Some(parent) = std::path::Path::new(&write_params.path).parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    std::fs::write(&write_params.path, &write_params.content).map_err(|e| {
                        BridgeError::Engine(format!(
                            "Failed to write script {}: {}",
                            write_params.path, e
                        ))
                    })?;

                    let result = forge_protocol::ScriptWriteResult {
                        success: true,
                        diagnostics: None,
                    };
                    Ok(serde_json::to_value(result)?)
                }
                Err(syntax_err) => {
                    let result = forge_protocol::ScriptWriteResult {
                        success: false,
                        diagnostics: Some(vec![syntax_err]),
                    };
                    Ok(serde_json::to_value(result)?)
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::create_bridge_channel;

    #[tokio::test]
    async fn test_script_write_rejects_invalid_syntax() {
        let (sender, _receiver) = create_bridge_channel();
        let invalid_code = "export function broken() { return {{{ invalid; }";
        let params = json!({
            "path": "test_invalid.ts",
            "content": invalid_code
        });

        let res = dispatch_method("script.write", params, &sender)
            .await
            .expect("dispatch");
        let write_result: forge_protocol::ScriptWriteResult =
            serde_json::from_value(res).expect("deserialize ScriptWriteResult");

        assert!(
            !write_result.success,
            "script.write must reject invalid syntax"
        );
        assert!(write_result.diagnostics.is_some());
        assert!(!write_result.diagnostics.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_script_write_accepts_valid_syntax() {
        let (sender, _receiver) = create_bridge_channel();
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let script_path = temp_dir
            .path()
            .join("valid_script.ts")
            .to_string_lossy()
            .to_string();
        let valid_code = "export function onStart(): void { const x: number = 42; }";
        let params = json!({
            "path": script_path,
            "content": valid_code
        });

        let res = dispatch_method("script.write", params, &sender)
            .await
            .expect("dispatch");
        let write_result: forge_protocol::ScriptWriteResult =
            serde_json::from_value(res).expect("deserialize ScriptWriteResult");

        assert!(write_result.success);
        assert!(std::path::Path::new(&script_path).exists());
    }
}
