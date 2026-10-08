// Forge Studio Tauri Application Library
pub mod runtime;

use runtime::*;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let runtime_manager = RuntimeManager::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(runtime_manager)
        .invoke_handler(tauri::generate_handler![
            launch_runtime,
            stop_runtime,
            restart_runtime,
            get_runtime_status,
            update_viewport_bounds,
            focus_viewport,
            focus_studio,
            set_viewport_mode,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                let manager = window.state::<RuntimeManager>();
                manager.stop();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Forge Studio application");
}
