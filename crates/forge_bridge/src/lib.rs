//! IPC bridge communicating between Rust engine and IDE host over WebSocket JSON-RPC 2.0 / MCP.

#![allow(clippy::result_large_err)]

pub mod command;
pub mod dispatcher;
pub mod error;
pub mod mcp;
pub mod server;

pub use command::{
    create_bridge_channel, BridgeCommand, BridgeCommandReceiver, BridgeCommandSender,
};
pub use error::BridgeError;
pub use mcp::{get_mcp_tools, McpTool, McpToolCallParams, McpToolCallResult, McpToolsListResult};
pub use server::{generate_auth_token, BridgeBroadcaster, BridgeServer};
