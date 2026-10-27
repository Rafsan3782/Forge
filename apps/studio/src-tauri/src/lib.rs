// Forge Studio Tauri Application Library
pub mod agent_host;
pub mod runtime;

use agent_host::*;
use runtime::*;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let runtime_manager = RuntimeManager::new();
    let agent_host_manager = AgentHostManager::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let agent_mgr = app.state::<AgentHostManager>();
            agent_mgr.set_app_handle(handle);
            Ok(())
        })
        .manage(runtime_manager)
        .manage(agent_host_manager)
        .invoke_handler(tauri::generate_handler![
            launch_runtime,
            stop_runtime,
            restart_runtime,
            get_runtime_status,
            update_viewport_bounds,
            focus_viewport,
            focus_studio,
            set_viewport_mode,
            window_minimize,
            window_toggle_maximize,
            window_close,
            window_start_dragging,
            start_agent,
            stop_agent,
            restart_agent,
            get_agent_status,
            update_agent_bridge,
            send_agent_command,
            set_api_key,
            has_api_key,
            delete_api_key,
            read_agent_settings,
            write_agent_settings,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                let manager = window.state::<RuntimeManager>();
                manager.stop();
                let agent_mgr = window.state::<AgentHostManager>();
                agent_mgr.stop();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Forge Studio application");
}
