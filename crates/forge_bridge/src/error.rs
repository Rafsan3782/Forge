//! Error types for the forge_bridge subsystem.

use forge_protocol::JsonRpcErrorDetail;
use thiserror::Error;

/// Errors that can occur within the Forge Bridge server.
#[derive(Debug, Error)]
pub enum BridgeError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization or deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// WebSocket transport error.
    #[error("WebSocket error: {0}")]
    Tungstenite(Box<tokio_tungstenite::tungstenite::Error>),

    /// Authentication failed or token missing/invalid.
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Method not found.
    #[error("Method not found: {0}")]
    MethodNotFound(String),

    /// Invalid parameters passed to method.
    #[error("Invalid params: {0}")]
    InvalidParams(String),

    /// Error returned by engine operation.
    #[error("Engine error: {0}")]
    Engine(String),

    /// Communication channel between bridge and engine closed.
    #[error("Engine channel closed")]
    ChannelClosed,
}

impl From<tokio_tungstenite::tungstenite::Error> for BridgeError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        BridgeError::Tungstenite(Box::new(err))
    }
}

impl BridgeError {
    /// Converts this error into a JSON-RPC 2.0 error detail struct.
    pub fn to_json_rpc_error(&self) -> JsonRpcErrorDetail {
        match self {
            BridgeError::Unauthorized(msg) => JsonRpcErrorDetail {
                code: -32001,
                message: format!("Unauthorized: {}", msg),
                data: None,
            },
            BridgeError::MethodNotFound(msg) => JsonRpcErrorDetail {
                code: -32601,
                message: format!("Method not found: {}", msg),
                data: None,
            },
            BridgeError::InvalidParams(msg) => JsonRpcErrorDetail {
                code: -32602,
                message: format!("Invalid params: {}", msg),
                data: None,
            },
            BridgeError::Engine(msg) => JsonRpcErrorDetail {
                code: -32000,
                message: msg.clone(),
                data: None,
            },
            BridgeError::ChannelClosed => JsonRpcErrorDetail {
                code: -32002,
                message: "Engine shutting down or channel closed".to_string(),
                data: None,
            },
            other => JsonRpcErrorDetail {
                code: -32603,
                message: other.to_string(),
                data: None,
            },
        }
    }
}
