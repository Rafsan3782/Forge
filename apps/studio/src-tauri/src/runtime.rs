use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, WebviewWindow};

#[cfg(windows)]
pub mod win32 {
    use std::ffi::c_void;

    pub type HWND = *mut c_void;
    pub type BOOL = i32;

    pub const HWND_TOP: HWND = 0 as HWND;
    pub const SWP_NOACTIVATE: u32 = 0x0010;
    pub const SWP_SHOWWINDOW: u32 = 0x0040;
    pub const SW_HIDE: i32 = 0;
    pub const SW_SHOW: i32 = 5;

    extern "system" {
        pub fn SetWindowPos(
            hWnd: HWND,
            hWndInsertAfter: HWND,
            X: i32,
            Y: i32,
            cx: i32,
            cy: i32,
            uFlags: u32,
        ) -> BOOL;
        pub fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
        pub fn SetFocus(hWnd: HWND) -> HWND;
        pub fn IsWindow(hWnd: HWND) -> BOOL;
        pub fn GetDpiForWindow(hwnd: HWND) -> u32;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeStatus {
    Stopped,
    Starting,
    Running,
    Crashed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub status: RuntimeStatus,
    pub port: Option<u16>,
    pub token: Option<String>,
    pub child_hwnd: Option<isize>,
    pub pid: Option<u32>,
    pub scene: String,
    pub mode: String, // "embed" or "stream"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeReadyPayload {
    pub port: u16,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCrashedPayload {
    pub exit_code: Option<i32>,
    pub reason: String,
}

struct InnerRuntimeState {
    child: Option<Child>,
    status: RuntimeStatus,
    port: Option<u16>,
    token: Option<String>,
    child_hwnd: Option<isize>,
    parent_hwnd: Option<isize>,
    pid: Option<u32>,
    current_scene: String,
    mode: String,
    last_bounds: Option<ViewportBounds>,
}

#[derive(Clone)]
pub struct RuntimeManager {
    inner: Arc<Mutex<InnerRuntimeState>>,
}

impl RuntimeManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(InnerRuntimeState {
                child: None,
                status: RuntimeStatus::Stopped,
                port: None,
                token: None,
                child_hwnd: None,
                parent_hwnd: None,
                pid: None,
                current_scene: "templates/hello/scene.json".to_string(),
                mode: "embed".to_string(),
                last_bounds: None,
            })),
        }
    }

    pub fn get_info(&self) -> RuntimeInfo {
        let guard = self.inner.lock().unwrap();
        RuntimeInfo {
            status: guard.status.clone(),
            port: guard.port,
            token: guard.token.clone(),
            child_hwnd: guard.child_hwnd,
            pid: guard.pid,
            scene: guard.current_scene.clone(),
            mode: guard.mode.clone(),
        }
    }

    pub fn set_mode(&self, mode: String) {
        let mut guard = self.inner.lock().unwrap();
        guard.mode = mode;
    }

    pub fn stop(&self) {
        let mut guard = self.inner.lock().unwrap();
        if let Some(mut child) = guard.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        guard.status = RuntimeStatus::Stopped;
        guard.port = None;
        guard.token = None;
        guard.child_hwnd = None;
        guard.pid = None;
    }

    pub fn update_bounds(&self, window: &WebviewWindow, bounds: ViewportBounds) {
        let (child_hwnd, mode) = {
            let mut guard = self.inner.lock().unwrap();
            guard.last_bounds = Some(bounds.clone());
            (guard.child_hwnd, guard.mode.clone())
        };

        if mode != "embed" {
            return;
        }

        #[cfg(windows)]
        if let Some(hwnd_val) = child_hwnd {
            let hwnd = hwnd_val as win32::HWND;
            unsafe {
                if win32::IsWindow(hwnd) == 0 {
                    return;
                }
            }

            if !bounds.visible || bounds.width <= 1.0 || bounds.height <= 1.0 {
                unsafe {
                    win32::ShowWindow(hwnd, win32::SW_HIDE);
                }
                return;
            }

            // High-DPI physical coordinates translation
            let scale = window.scale_factor().unwrap_or(1.0);
            let px = (bounds.x * scale).round() as i32;
            let py = (bounds.y * scale).round() as i32;
            let pw = (bounds.width * scale).round() as i32;
            let ph = (bounds.height * scale).round() as i32;

            unsafe {
                win32::ShowWindow(hwnd, win32::SW_SHOW);
                win32::SetWindowPos(
                    hwnd,
                    win32::HWND_TOP,
                    px,
                    py,
                    pw,
                    ph,
                    win32::SWP_SHOWWINDOW | win32::SWP_NOACTIVATE,
                );
            }
        }
    }

    pub fn focus_child(&self) {
        #[cfg(windows)]
        {
            let hwnd_opt = {
                let guard = self.inner.lock().unwrap();
                guard.child_hwnd
            };
            if let Some(h) = hwnd_opt {
                unsafe {
                    win32::SetFocus(h as win32::HWND);
                }
            }
        }
    }

    pub fn focus_parent(&self) {
        #[cfg(windows)]
        {
            let hwnd_opt = {
                let guard = self.inner.lock().unwrap();
                guard.parent_hwnd
            };
            if let Some(h) = hwnd_opt {
                unsafe {
                    win32::SetFocus(h as win32::HWND);
                }
            }
        }
    }
}

fn find_runtime_binary() -> Result<PathBuf, String> {
    // 1. Current working directory
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let candidates = [
        cwd.join("target").join("debug").join("forge_runtime.exe"),
        cwd.join("..").join("..").join("target").join("debug").join("forge_runtime.exe"),
        cwd.join("target").join("release").join("forge_runtime.exe"),
        cwd.join("..").join("..").join("target").join("release").join("forge_runtime.exe"),
    ];

    for cand in &candidates {
        if cand.exists() {
            return Ok(cand.canonicalize().unwrap_or_else(|_| cand.clone()));
        }
    }

    // 2. Walk up to workspace root
    let mut cur = cwd.clone();
    for _ in 0..6 {
        let direct = cur.join("target").join("debug").join("forge_runtime.exe");
        if direct.exists() {
            return Ok(direct.canonicalize().unwrap_or_else(|_| direct));
        }
        if !cur.pop() {
            break;
        }
    }

    Err(format!(
        "forge_runtime binary not found. Checked: {:?}",
        candidates
    ))
}

#[tauri::command]
pub async fn launch_runtime(
    app: AppHandle,
    window: WebviewWindow,
    manager: tauri::State<'_, RuntimeManager>,
    scene: Option<String>,
    mode: Option<String>,
) -> Result<RuntimeInfo, String> {
    // Stop any existing instance
    manager.stop();

    let target_scene = scene.unwrap_or_else(|| "templates/hello/scene.json".to_string());
    let target_mode = mode.unwrap_or_else(|| "embed".to_string());

    #[cfg(windows)]
    let parent_hwnd: isize = match window.hwnd() {
        Ok(h) => h.0 as isize,
        Err(e) => return Err(format!("Failed to retrieve Studio window HWND: {e}")),
    };

    #[cfg(not(windows))]
    let parent_hwnd: isize = 0;

    let runtime_path = find_runtime_binary()?;

    let mut cmd = Command::new(&runtime_path);
    cmd.arg("--scene").arg(&target_scene);

    if target_mode == "stream" {
        cmd.arg("--headless").arg("--stay-open");
    } else {
        #[cfg(windows)]
        cmd.arg("--embed-parent").arg(format!("{}", parent_hwnd));
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // Working directory set to repo root if possible
    if let Ok(root) = std::env::current_dir() {
        cmd.current_dir(root);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn forge_runtime ({}): {e}", runtime_path.display()))?;

    let pid = child.id();
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;

    {
        let mut guard = manager.inner.lock().unwrap();
        guard.child = Some(child);
        guard.status = RuntimeStatus::Starting;
        guard.pid = Some(pid);
        guard.parent_hwnd = Some(parent_hwnd);
        guard.current_scene = target_scene.clone();
        guard.mode = target_mode.clone();
    }

    let manager_clone = manager.inner.clone();
    let app_clone = app.clone();
    let window_clone = window.clone();

    // Spawn background stdout listener thread to parse FORGE_BRIDGE_READY and FORGE_WINDOW_READY
    std::thread::Builder::new()
        .name("forge-runtime-monitor".to_string())
        .spawn(move || {
            let reader = BufReader::new(stdout);
            for line_res in reader.lines() {
                let line = match line_res {
                    Ok(l) => l,
                    Err(_) => break,
                };

                if line.starts_with("FORGE_BRIDGE_READY") {
                    if let Some(json_str) = line.strip_prefix("FORGE_BRIDGE_READY").map(str::trim) {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) {
                            let port = v.get("port").and_then(|p| p.as_u64()).map(|p| p as u16);
                            let token = v.get("token").and_then(|t| t.as_str()).map(ToString::to_string);

                            if let (Some(p), Some(t)) = (port, token) {
                                {
                                    let mut guard = manager_clone.lock().unwrap();
                                    guard.port = Some(p);
                                    guard.token = Some(t.clone());
                                    guard.status = RuntimeStatus::Running;
                                }

                                let _ = app_clone.emit("forge://bridge-ready", BridgeReadyPayload {
                                    port: p,
                                    token: t,
                                });

                                let _ = app_clone.emit("forge://runtime-status", "running");
                            }
                        }
                    }
                } else if line.starts_with("FORGE_WINDOW_READY") {
                    if let Some(json_str) = line.strip_prefix("FORGE_WINDOW_READY").map(str::trim) {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) {
                            if let Some(h) = v.get("hwnd").and_then(|h| h.as_i64()) {
                                let hwnd = h as isize;
                                let bounds_opt = {
                                    let mut guard = manager_clone.lock().unwrap();
                                    guard.child_hwnd = Some(hwnd);
                                    guard.last_bounds.clone()
                                };

                                // If viewport bounds were already recorded, immediately apply them
                                if let Some(bounds) = bounds_opt {
                                    #[cfg(windows)]
                                    {
                                        let scale = window_clone.scale_factor().unwrap_or(1.0);
                                        let px = (bounds.x * scale).round() as i32;
                                        let py = (bounds.y * scale).round() as i32;
                                        let pw = (bounds.width * scale).round() as i32;
                                        let ph = (bounds.height * scale).round() as i32;

                                        unsafe {
                                            win32::ShowWindow(hwnd as win32::HWND, win32::SW_SHOW);
                                            win32::SetWindowPos(
                                                hwnd as win32::HWND,
                                                win32::HWND_TOP,
                                                px,
                                                py,
                                                pw,
                                                ph,
                                                win32::SWP_SHOWWINDOW | win32::SWP_NOACTIVATE,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // When stdout closes, inspect process exit status
            let mut exit_code = None;
            let mut was_expected_stop = false;

            {
                let mut guard = manager_clone.lock().unwrap();
                if let Some(mut child) = guard.child.take() {
                    if let Ok(status) = child.wait() {
                        exit_code = status.code();
                    }
                }
                if guard.status == RuntimeStatus::Stopped {
                    was_expected_stop = true;
                } else {
                    guard.status = RuntimeStatus::Crashed;
                    guard.port = None;
                    guard.token = None;
                    guard.child_hwnd = None;
                    guard.pid = None;
                }
            }

            if !was_expected_stop {
                let _ = app_clone.emit("forge://runtime-crashed", RuntimeCrashedPayload {
                    exit_code,
                    reason: format!("Engine process exited unexpectedly with code {:?}", exit_code),
                });
                let _ = app_clone.emit("forge://runtime-status", "crashed");
            } else {
                let _ = app_clone.emit("forge://runtime-status", "stopped");
            }
        })
        .map_err(|e| format!("Failed to spawn monitor thread: {e}"))?;

    Ok(manager.get_info())
}

#[tauri::command]
pub fn stop_runtime(manager: tauri::State<'_, RuntimeManager>) -> Result<(), String> {
    manager.stop();
    Ok(())
}

#[tauri::command]
pub async fn restart_runtime(
    app: AppHandle,
    window: WebviewWindow,
    manager: tauri::State<'_, RuntimeManager>,
    scene: Option<String>,
    mode: Option<String>,
) -> Result<RuntimeInfo, String> {
    launch_runtime(app, window, manager, scene, mode).await
}

#[tauri::command]
pub fn get_runtime_status(manager: tauri::State<'_, RuntimeManager>) -> RuntimeInfo {
    manager.get_info()
}

#[tauri::command]
pub fn update_viewport_bounds(
    window: WebviewWindow,
    manager: tauri::State<'_, RuntimeManager>,
    bounds: ViewportBounds,
) -> Result<(), String> {
    manager.update_bounds(&window, bounds);
    Ok(())
}

#[tauri::command]
pub fn focus_viewport(manager: tauri::State<'_, RuntimeManager>) -> Result<(), String> {
    manager.focus_child();
    Ok(())
}

#[tauri::command]
pub fn focus_studio(manager: tauri::State<'_, RuntimeManager>) -> Result<(), String> {
    manager.focus_parent();
    Ok(())
}

#[tauri::command]
pub fn set_viewport_mode(
    manager: tauri::State<'_, RuntimeManager>,
    mode: String,
) -> Result<(), String> {
    manager.set_mode(mode);
    Ok(())
}
