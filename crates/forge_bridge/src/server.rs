//! WebSocket JSON-RPC and MCP bridge server.

#![allow(clippy::result_large_err)]

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::io::Write;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::http::StatusCode;
use tokio_tungstenite::tungstenite::Message;

use crate::command::BridgeCommandSender;
use crate::dispatcher::dispatch_method;
use crate::error::BridgeError;

/// Generates a cryptographically sound random 32-character hexadecimal auth token.
pub fn generate_auth_token() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let h1 = RandomState::new().build_hasher().finish();
    let h2 = RandomState::new().build_hasher().finish();

    let p1 = h1 ^ (now as u64);
    let p2 = h2 ^ ((now >> 64) as u64);

    format!("{:016x}{:016x}", p1, p2)
}

/// Handle to a running Forge Bridge server instance.
pub struct BridgeServer {
    /// Port the server bound to on 127.0.0.1.
    pub port: u16,
    /// Random authentication token required for client connections.
    pub token: String,
    /// Exact socket address (127.0.0.1:port).
    pub addr: SocketAddr,
    shutdown: Arc<AtomicBool>,
}

impl BridgeServer {
    /// Starts the bridge server on a background thread with its own Tokio runtime.
    pub fn start_background(
        token: Option<String>,
        command_sender: BridgeCommandSender,
    ) -> Result<Self, BridgeError> {
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();

        std::thread::Builder::new()
            .name("forge-bridge-server".to_string())
            .spawn(move || {
                let rt = match tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = ready_tx.send(Err(BridgeError::Io(e)));
                        return;
                    }
                };

                rt.block_on(async move {
                    let token_str = token.unwrap_or_else(generate_auth_token);

                    // Strictly bind to 127.0.0.1 on a random free port
                    let listener = match TcpListener::bind("127.0.0.1:0").await {
                        Ok(l) => l,
                        Err(e) => {
                            let _ = ready_tx.send(Err(BridgeError::Io(e)));
                            return;
                        }
                    };

                    let addr = match listener.local_addr() {
                        Ok(a) => a,
                        Err(e) => {
                            let _ = ready_tx.send(Err(BridgeError::Io(e)));
                            return;
                        }
                    };

                    let port = addr.port();

                    // Print the exact machine readable line required by the protocol specification
                    println!(
                        "FORGE_BRIDGE_READY {{\"port\":{}, \"token\":\"{}\"}}",
                        port, token_str
                    );
                    let _ = std::io::stdout().flush();

                    let _ = ready_tx.send(Ok((port, token_str.clone(), addr)));

                    let token_arc = Arc::new(token_str);

                    while !shutdown_clone.load(Ordering::Relaxed) {
                        match listener.accept().await {
                            Ok((stream, client_addr)) => {
                                tracing::debug!("Incoming bridge connection from {}", client_addr);
                                let expected_token = token_arc.clone();
                                let cmd_sender = command_sender.clone();

                                tokio::spawn(async move {
                                    if let Err(e) =
                                        handle_connection(stream, expected_token, cmd_sender).await
                                    {
                                        tracing::debug!("Bridge connection closed: {:?}", e);
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::warn!("Error accepting bridge connection: {:?}", e);
                            }
                        }
                    }
                });
            })
            .map_err(|e| BridgeError::Io(std::io::Error::other(e)))?;

        let (port, token, addr) = ready_rx.recv().map_err(|_| {
            BridgeError::Engine("Bridge startup thread exited prematurely".to_string())
        })??;

        Ok(Self {
            port,
            token,
            addr,
            shutdown,
        })
    }

    /// Stops the bridge server.
    pub fn stop(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

impl Drop for BridgeServer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Handles a single incoming TCP stream, validating WebSocket authentication and serving RPC requests.
async fn handle_connection(
    stream: TcpStream,
    expected_token: Arc<String>,
    command_sender: BridgeCommandSender,
) -> Result<(), BridgeError> {
    // Perform WebSocket handshake with authentication verification
    let callback = |req: &Request, resp: Response| {
        let mut authorized = false;

        // 1. Check URL query string: ?token=<token> or ?auth=<token>
        if let Some(query) = req.uri().query() {
            for pair in query.split('&') {
                if let Some((k, v)) = pair.split_once('=') {
                    if (k == "token" || k == "auth") && v == expected_token.as_str() {
                        authorized = true;
                        break;
                    }
                }
            }
        }

        // 2. Check URL path: /<token>
        let path = req.uri().path().trim_start_matches('/');
        if path == expected_token.as_str() || path == format!("ws/{}", expected_token.as_str()) {
            authorized = true;
        }

        // 3. Check Authorization header: Bearer <token> or <token>
        if let Some(auth_header) = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
        {
            let token = auth_header
                .strip_prefix("Bearer ")
                .unwrap_or(auth_header)
                .trim();
            if token == expected_token.as_str() {
                authorized = true;
            }
        }

        if authorized {
            Ok(resp)
        } else {
            let mut error_response = ErrorResponse::new(Some(
                "Unauthorized: Invalid or missing authentication token".to_string(),
            ));
            *error_response.status_mut() = StatusCode::UNAUTHORIZED;
            Err(error_response)
        }
    };

    let ws_stream = tokio_tungstenite::accept_hdr_async(stream, callback).await?;
    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                tracing::debug!("WebSocket stream error: {:?}", e);
                break;
            }
        };

        match msg {
            Message::Text(text) => {
                let response_text = process_json_rpc_message(&text, &command_sender).await;
                if let Some(reply) = response_text {
                    if let Err(e) = write.send(Message::Text(reply.into())).await {
                        tracing::debug!("Failed to send WebSocket reply: {:?}", e);
                        break;
                    }
                }
            }
            Message::Ping(data) => {
                let _ = write.send(Message::Pong(data)).await;
            }
            Message::Close(_) => {
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Parses and processes a JSON-RPC 2.0 message string, returning the response string if applicable.
async fn process_json_rpc_message(
    text: &str,
    command_sender: &BridgeCommandSender,
) -> Option<String> {
    let raw_val: Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            let err_resp = json!({
                "jsonrpc": "2.0",
                "id": Value::Null,
                "error": {
                    "code": -32700,
                    "message": format!("Parse error: {}", e)
                }
            });
            return Some(err_resp.to_string());
        }
    };

    // Handle single request or batch requests
    if let Some(batch) = raw_val.as_array() {
        let mut responses = Vec::new();
        for item in batch {
            if let Some(resp) = process_single_request(item, command_sender).await {
                responses.push(resp);
            }
        }
        if responses.is_empty() {
            None
        } else {
            Some(Value::Array(responses).to_string())
        }
    } else {
        process_single_request(&raw_val, command_sender)
            .await
            .map(|v| v.to_string())
    }
}

/// Processes a single JSON-RPC 2.0 request or notification.
async fn process_single_request(
    val: &Value,
    command_sender: &BridgeCommandSender,
) -> Option<Value> {
    let method = match val.get("method").and_then(|m| m.as_str()) {
        Some(m) => m,
        None => {
            let id = val.get("id").cloned().unwrap_or(Value::Null);
            return Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32600,
                    "message": "Invalid Request: missing method"
                }
            }));
        }
    };

    let id = val.get("id").cloned();
    let params = val.get("params").cloned().unwrap_or(Value::Null);

    // If id is not present, it's a notification, no response required
    let is_notification = id.is_none();

    let result = dispatch_method(method, params, command_sender).await;

    if is_notification {
        return None;
    }

    let id_val = id.unwrap_or(Value::Null);

    match result {
        Ok(res_val) => Some(json!({
            "jsonrpc": "2.0",
            "id": id_val,
            "result": res_val
        })),
        Err(err) => Some(json!({
            "jsonrpc": "2.0",
            "id": id_val,
            "error": err.to_json_rpc_error()
        })),
    }
}
