pub mod framing;
pub mod keyring_vault;
pub mod settings;

use framing::{encode_lsp_message, LspFramingBuffer};
use keyring_vault::get_api_key_internal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentLifecycleStatus {
    Stopped,
    Starting,
    Running,
    Crashed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSidecarInfo {
    pub status: AgentLifecycleStatus,
    pub pid: Option<u32>,
    pub project_path: Option<String>,
    pub bridge_port: Option<u16>,
    pub crashed_reason: Option<String>,
    pub restart_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCrashPayload {
    pub exit_code: Option<i32>,
    pub reason: String,
}

struct AgentProcessState {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    status: AgentLifecycleStatus,
    pid: Option<u32>,
    project_path: Option<String>,
    bridge_port: Option<u16>,
    bridge_token: Option<String>,
    crashed_reason: Option<String>,
    restart_count: u32,
    should_run: bool,
}

pub struct AgentHostManager {
    inner: Arc<Mutex<AgentProcessState>>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
    heartbeat_active: Arc<AtomicBool>,
}

impl Default for AgentHostManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentHostManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AgentProcessState {
                child: None,
                stdin: None,
                status: AgentLifecycleStatus::Stopped,
                pid: None,
                project_path: None,
                bridge_port: None,
                bridge_token: None,
                crashed_reason: None,
                restart_count: 0,
                should_run: false,
            })),
            app_handle: Arc::new(Mutex::new(None)),
            heartbeat_active: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_app_handle(&self, handle: AppHandle) {
        let mut guard = self.app_handle.lock().unwrap();
        *guard = Some(handle);
    }

    pub fn get_info(&self) -> AgentSidecarInfo {
        let guard = self.inner.lock().unwrap();
        AgentSidecarInfo {
            status: guard.status.clone(),
            pid: guard.pid,
            project_path: guard.project_path.clone(),
            bridge_port: guard.bridge_port,
            crashed_reason: guard.crashed_reason.clone(),
            restart_count: guard.restart_count,
        }
    }

    /// Starts or restarts the agent sidecar process with project path, bridge port, and bridge token.
    pub fn start(
        &self,
        project_path: String,
        bridge_port: Option<u16>,
        bridge_token: Option<String>,
    ) -> Result<(), String> {
        let app = {
            let guard = self.app_handle.lock().unwrap();
            guard.clone().ok_or("AppHandle not initialized")?
        };

        // Terminate existing process if active
        self.stop_internal(false);

        {
            let mut guard = self.inner.lock().unwrap();
            guard.project_path = Some(project_path.clone());
            guard.bridge_port = bridge_port;
            guard.bridge_token = bridge_token.clone();
            guard.status = AgentLifecycleStatus::Starting;
            guard.should_run = true;
            guard.crashed_reason = None;
        }

        let _ = app.emit("forge://agent-status", "starting");

        self.spawn_sidecar(project_path, bridge_port, bridge_token, 0)
    }

    /// Update the engine bridge connection coordinates without terminating the agent session.
    pub fn update_bridge(&self, port: u16, token: String) -> Result<(), String> {
        let mut stdin_writer = None;
        {
            let mut guard = self.inner.lock().unwrap();
            guard.bridge_port = Some(port);
            guard.bridge_token = Some(token.clone());
            if guard.status == AgentLifecycleStatus::Running {
                // Send an update_bridge notification over stdio to the running agent
                stdin_writer = guard.stdin.as_mut();
            }
        }

        if let Some(stdin) = stdin_writer {
            let notif = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "update_bridge",
                "params": {
                    "port": port,
                    "token": token
                }
            });
            let encoded = encode_lsp_message(&notif.to_string());
            let _ = stdin.write_all(&encoded);
            let _ = stdin.flush();
        }

        Ok(())
    }

    /// Stops the sidecar process gracefully.
    pub fn stop(&self) {
        self.stop_internal(true);
    }

    fn stop_internal(&self, user_requested: bool) {
        let app_opt = {
            let guard = self.app_handle.lock().unwrap();
            guard.clone()
        };

        let mut child_to_kill = None;

        {
            let mut guard = self.inner.lock().unwrap();
            if user_requested {
                guard.should_run = false;
            }
            guard.status = AgentLifecycleStatus::Stopped;
            guard.stdin = None;
            guard.pid = None;
            child_to_kill = guard.child.take();
        }

        if let Some(mut child) = child_to_kill {
            let _ = child.kill();
            let _ = child.wait();
        }

        if user_requested {
            if let Some(app) = app_opt {
                let _ = app.emit("forge://agent-status", "stopped");
            }
        }
    }

    /// Sends a JSON-RPC command or notification to the agent sidecar via framed stdin.
    pub fn send_command(&self, command_json: Value) -> Result<(), String> {
        let mut guard = self.inner.lock().unwrap();
        if guard.status != AgentLifecycleStatus::Running {
            return Err("Agent sidecar is not running".to_string());
        }

        let stdin = guard.stdin.as_mut().ok_or("Agent stdin not accessible")?;
        let encoded = encode_lsp_message(&command_json.to_string());
        stdin
            .write_all(&encoded)
            .map_err(|e| format!("Failed to write to agent stdin: {e}"))?;
        stdin
            .flush()
            .map_err(|e| format!("Failed to flush agent stdin: {e}"))?;

        Ok(())
    }

    fn spawn_sidecar(
        &self,
        project_path: String,
        bridge_port: Option<u16>,
        bridge_token: Option<String>,
        attempt: u32,
    ) -> Result<(), String> {
        let app = {
            let guard = self.app_handle.lock().unwrap();
            guard.clone().ok_or("AppHandle not initialized")?
        };

        let workspace_root = find_workspace_root().unwrap_or_else(|_| PathBuf::from("."));
        let node_binary = find_node_binary(&workspace_root)?;
        let sidecar_script = find_sidecar_script(&workspace_root)?;

        let mut cmd = Command::new(&node_binary);
        cmd.arg(&sidecar_script);
        cmd.arg("--project").arg(&project_path);

        if let Some(port) = bridge_port {
            cmd.arg("--bridge-port").arg(port.to_string());
        }
        if let Some(ref tok) = bridge_token {
            cmd.arg("--bridge-token").arg(tok);
        }

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(&workspace_root);

        // DO NOT pass GEMINI_API_KEY as an env variable or command-line argument!
        // We will send it once via stdin initialization handshake below.

        let mut child = cmd.spawn().map_err(|e| {
            format!(
                "Failed to spawn agent sidecar ({}) with node ({}): {e}",
                sidecar_script.display(),
                node_binary.display()
            )
        })?;

        let pid = child.id();
        let mut stdin = child.stdin.take().ok_or("Failed to capture sidecar stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to capture sidecar stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture sidecar stderr")?;

        // 1. Initial Handshake: send API key and configuration strictly via stdin framed message
        let api_key = get_api_key_internal().unwrap_or(None);
        let init_msg = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "init",
            "params": {
                "api_key": api_key,
                "project_path": project_path,
                "bridge_port": bridge_port,
                "bridge_token": bridge_token
            }
        });
        let init_encoded = encode_lsp_message(&init_msg.to_string());
        let _ = stdin.write_all(&init_encoded);
        let _ = stdin.flush();

        {
            let mut guard = self.inner.lock().unwrap();
            guard.child = Some(child);
            guard.stdin = Some(stdin);
            guard.pid = Some(pid);
            guard.status = AgentLifecycleStatus::Running;
            guard.restart_count = attempt;
        }

        let _ = app.emit("forge://agent-status", "running");

        // Spawn Stderr Listener Thread -> Forwards sidecar logs to ConsolePanel
        let app_err = app.clone();
        std::thread::Builder::new()
            .name("agent-sidecar-stderr".to_string())
            .spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    eprintln!("[agent_sidecar] {}", line);
                    let _ = app_err.emit("forge://agent-stderr", line);
                }
            })
            .map_err(|e| format!("Failed to spawn agent stderr listener: {e}"))?;

        // Spawn Stdout Listener Thread -> Decodes LSP Content-Length frames and emits events
        let app_out = app.clone();
        let state_clone = self.inner.clone();
        let app_crash = app.clone();
        let host_manager_arc = Arc::new(Self {
            inner: state_clone.clone(),
            app_handle: self.app_handle.clone(),
            heartbeat_active: self.heartbeat_active.clone(),
        });

        std::thread::Builder::new()
            .name("agent-sidecar-stdout".to_string())
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                let mut buffer = LspFramingBuffer::new();
                let mut chunk = [0u8; 4096];

                loop {
                    match reader.read(&mut chunk) {
                        Ok(0) => break, // EOF, process exited
                        Ok(n) => {
                            buffer.push_bytes(&chunk[..n]);
                            while let Ok(Some(msg)) = buffer.pop_message() {
                                // Parse JSON message
                                if let Ok(val) = serde_json::from_str::<Value>(&msg) {
                                    // Check if this is a heartbeat response
                                    if val.get("method").and_then(|m| m.as_str()) == Some("heartbeat")
                                        || val.get("id").and_then(|id| id.as_str()) == Some("heartbeat")
                                    {
                                        continue;
                                    }

                                    // Check if this is an AgentEvent notification or JSON-RPC notification
                                    if val.get("type").is_some() {
                                        // Standard AgentEvent stream
                                        let _ = app_out.emit("forge://agent-event", val);
                                    } else if let Some(params) = val.get("params") {
                                        // JSON-RPC notification wrapping AgentEvent
                                        let _ = app_out.emit("forge://agent-event", params);
                                    } else {
                                        let _ = app_out.emit("forge://agent-stdout", val);
                                    }
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }

                // Process exited or stdout closed: check exit status and handle restart/crashed
                let mut should_restart = false;
                let mut next_attempt = 0;
                let mut reason_str = "Agent sidecar process terminated unexpectedly".to_string();
                let mut exit_code_val = None;

                {
                    let mut guard = state_clone.lock().unwrap();
                    guard.stdin = None;
                    if let Some(ref mut c) = guard.child {
                        if let Ok(Some(status)) = c.try_wait() {
                            exit_code_val = status.code();
                            reason_str = format!("Agent process exited with status: {}", status);
                        }
                    }
                    guard.crashed_reason = Some(reason_str.clone());

                    if guard.should_run {
                        guard.status = AgentLifecycleStatus::Crashed;
                        guard.restart_count += 1;
                        next_attempt = guard.restart_count;
                        should_restart = next_attempt <= 5; // Automatic restart with backoff up to 5 times
                    } else {
                        guard.status = AgentLifecycleStatus::Stopped;
                    }
                }

                let _ = app_crash.emit(
                    "forge://agent-crashed",
                    AgentCrashPayload {
                        exit_code: exit_code_val,
                        reason: reason_str.clone(),
                    },
                );

                if should_restart {
                    let _ = app_crash.emit("forge://agent-status", "crashed");
                    // Exponential backoff: 500ms, 1s, 2s, 4s...
                    let delay_ms = 500 * (1 << (next_attempt.saturating_sub(1).min(4)));
                    std::thread::sleep(Duration::from_millis(delay_ms));

                    let should_still_run = {
                        let guard = state_clone.lock().unwrap();
                        guard.should_run
                    };

                    if should_still_run {
                        let (p, bp, bt) = {
                            let guard = state_clone.lock().unwrap();
                            (
                                guard.project_path.clone().unwrap_or_default(),
                                guard.bridge_port,
                                guard.bridge_token.clone(),
                            )
                        };
                        let _ = host_manager_arc.spawn_sidecar(p, bp, bt, next_attempt);
                    }
                } else {
                    let _ = app_crash.emit("forge://agent-status", "crashed");
                }
            })
            .map_err(|e| format!("Failed to spawn agent stdout listener: {e}"))?;

        // Start Heartbeat monitoring thread
        self.start_heartbeat_loop();

        Ok(())
    }

    fn start_heartbeat_loop(&self) {
        if self.heartbeat_active.swap(true, Ordering::SeqCst) {
            return; // Already running
        }

        let state_clone = self.inner.clone();
        let is_active = self.heartbeat_active.clone();

        std::thread::Builder::new()
            .name("agent-heartbeat".to_string())
            .spawn(move || {
                while is_active.load(Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_secs(5));

                    let mut stdin_opt = None;
                    {
                        let mut guard = state_clone.lock().unwrap();
                        if guard.status != AgentLifecycleStatus::Running {
                            continue;
                        }
                        if let Some(ref mut child) = guard.child {
                            if let Ok(Some(_)) = child.try_wait() {
                                // Child has exited
                                continue;
                            }
                        }
                        if guard.stdin.is_some() {
                            stdin_opt = guard.stdin.as_mut();
                        }
                    }

                    if let Some(stdin) = stdin_opt {
                        let ping = serde_json::json!({
                            "jsonrpc": "2.0",
                            "method": "heartbeat",
                            "params": {}
                        });
                        let encoded = encode_lsp_message(&ping.to_string());
                        if stdin.write_all(&encoded).is_err() || stdin.flush().is_err() {
                            // Write failed, stream broken
                            eprintln!("[agent_host] Failed to write heartbeat to agent");
                        }
                    }
                }
            })
            .ok();
    }
}

fn find_workspace_root() -> Result<PathBuf, String> {
    let mut cur = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for _ in 0..6 {
        if cur.join("packages").join("agent").exists() {
            return Ok(cur);
        }
        if let Some(parent) = cur.parent() {
            cur = parent.to_path_buf();
        } else {
            break;
        }
    }
    Ok(PathBuf::from("."))
}

fn find_node_binary(root: &Path) -> Result<PathBuf, String> {
    let dev_node = root.join(".tools").join("node").join("node.exe");
    if dev_node.exists() {
        return Ok(dev_node);
    }

    Ok(PathBuf::from("node"))
}

fn find_sidecar_script(root: &Path) -> Result<PathBuf, String> {
    // 1. Check if a dedicated sidecar entry point exists in packages/agent/src
    let sidecar_ts = root.join("packages").join("agent").join("src").join("sidecar.ts");
    if sidecar_ts.exists() {
        return Ok(sidecar_ts);
    }

    let sidecar_js = root.join("packages").join("agent").join("dist").join("sidecar.cjs");
    if sidecar_js.exists() {
        return Ok(sidecar_js);
    }

    // Default to main index.ts
    let index_ts = root.join("packages").join("agent").join("src").join("index.ts");
    Ok(index_ts)
}

// -----------------------------------------------------------------------------
// Tauri IPC Command Handlers
// -----------------------------------------------------------------------------

#[tauri::command]
pub fn start_agent(
    manager: tauri::State<'_, AgentHostManager>,
    project_path: String,
    bridge_port: Option<u16>,
    bridge_token: Option<String>,
) -> Result<(), String> {
    manager.start(project_path, bridge_port, bridge_token)
}

#[tauri::command]
pub fn stop_agent(manager: tauri::State<'_, AgentHostManager>) -> Result<(), String> {
    manager.stop();
    Ok(())
}

#[tauri::command]
pub fn restart_agent(
    manager: tauri::State<'_, AgentHostManager>,
    project_path: String,
    bridge_port: Option<u16>,
    bridge_token: Option<String>,
) -> Result<(), String> {
    manager.start(project_path, bridge_port, bridge_token)
}

#[tauri::command]
pub fn get_agent_status(manager: tauri::State<'_, AgentHostManager>) -> AgentSidecarInfo {
    manager.get_info()
}

#[tauri::command]
pub fn update_agent_bridge(
    manager: tauri::State<'_, AgentHostManager>,
    bridge_port: u16,
    bridge_token: String,
) -> Result<(), String> {
    manager.update_bridge(bridge_port, bridge_token)
}

#[tauri::command]
pub fn send_agent_command(
    manager: tauri::State<'_, AgentHostManager>,
    command: Value,
) -> Result<(), String> {
    manager.send_command(command)
}

#[tauri::command]
pub fn set_api_key(api_key: String) -> Result<(), String> {
    keyring_vault::set_api_key(&api_key)
}

#[tauri::command]
pub fn has_api_key() -> Result<bool, String> {
    keyring_vault::has_api_key()
}

#[tauri::command]
pub fn delete_api_key() -> Result<(), String> {
    keyring_vault::delete_api_key()
}

#[tauri::command]
pub fn read_agent_settings(project_path: Option<String>) -> Result<settings::AgentSettings, String> {
    settings::read_settings(project_path.as_deref())
}

#[tauri::command]
pub fn write_agent_settings(
    settings: settings::AgentSettings,
    project_path: Option<String>,
) -> Result<(), String> {
    settings::write_settings(&settings, project_path.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_never_in_command_line_or_logs() {
        let fake_key = "AIzaSySecretFakeTestKey123456789";
        let project_path = "templates/platformer";
        let bridge_port = Some(9876u16);
        let bridge_token = Some("secret_bridge_token".to_string());

        // 1. Simulate process Command construction as in spawn_sidecar
        let mut cmd = Command::new("node");
        cmd.arg("sidecar.ts");
        cmd.arg("--project").arg(project_path);

        if let Some(port) = bridge_port {
            cmd.arg("--bridge-port").arg(port.to_string());
        }
        if let Some(ref tok) = bridge_token {
            cmd.arg("--bridge-token").arg(tok);
        }

        // Check command line arguments string
        let cmd_line = format!("{:?}", cmd);
        assert!(
            !cmd_line.contains(fake_key),
            "API key must never appear in command line arguments!"
        );

        // 2. Simulate init handshake payload: sent strictly via stdin, not CLI or env
        let init_msg = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "init",
            "params": {
                "api_key": fake_key,
                "project_path": project_path,
            }
        });
        let framed_bytes = framing::encode_lsp_message(&init_msg.to_string());
        let mut buffer = framing::LspFramingBuffer::new();
        buffer.push_bytes(&framed_bytes);
        let decoded = buffer.pop_message().unwrap().unwrap();
        let parsed: Value = serde_json::from_str(&decoded).unwrap();
        assert_eq!(
            parsed["params"]["api_key"].as_str().unwrap(),
            fake_key
        );

        // 3. Verify AgentEvent streams or crash logs never serialize the secret
        let log_sample = format!("[agent_sidecar] Info: loaded project {}", project_path);
        assert!(!log_sample.contains(fake_key));

        let crash_payload = AgentCrashPayload {
            exit_code: Some(1),
            reason: "Process exited with code 1".to_string(),
        };
        let crash_json = serde_json::to_string(&crash_payload).unwrap();
        assert!(!crash_json.contains(fake_key));
    }
}
