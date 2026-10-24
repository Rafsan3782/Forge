use forge_core::components::{EntityId, Name, Script, Transform2D};
use forge_core::ecs::World;
use forge_core::events::EventBus;
use forge_core::time::Time;
use forge_protocol::ScriptErrorEvent;
use forge_script::{
    transpile_typescript, LoadedScript, SandboxConfig, ScriptExecutionError, ScriptRuntime,
    ScriptServer,
};
use std::time::Duration;

#[test]
fn test_lifecycle_order_and_ecs_mutation() {
    let script_source = r#"
        interface State {
            count: number;
        }
        let state: State = { count: 0 };

        export function onStart(): void {
            state.count += 10;
        }

        export function onUpdate(dt: number): void {
            state.count += 5;
            const t = forge.entity.getTransform();
            t.position[0] += 1.0;
            forge.entity.setTransform(t);
        }

        export function onDestroy(): void {
            state.count = 0;
        }
    "#;

    let mut runtime = ScriptRuntime::new(SandboxConfig::default()).expect("runtime init");
    let exec_context = forge_script::ScriptExecutionContext::new();
    forge_script::bind_forge_api(runtime.context(), exec_context).expect("bind api");
    let loaded = runtime
        .load_typescript_module("lifecycle.ts", script_source)
        .expect("load module");

    assert!(loaded.has_on_start);
    assert!(loaded.has_on_update);
    assert!(loaded.has_on_destroy);

    let start_res: Result<(), _> = runtime.invoke_function("e1", &loaded, "onStart", ());
    assert!(start_res.is_ok());

    let update_res: Result<(), _> = runtime.invoke_function("e1", &loaded, "onUpdate", (0.016f32,));
    assert!(update_res.is_ok());
}

#[test]
fn test_sandbox_limits_and_infinite_loop_interrupted() {
    let loop_script = r#"
        export function onUpdate(dt: number): void {
            while (true) {
                // Infinite busy loop exceeding frame duration
            }
        }
    "#;

    let mut config = SandboxConfig::default();
    config.max_duration_per_frame = Duration::from_millis(5);

    let mut runtime = ScriptRuntime::new(config).expect("runtime init");
    let loaded = runtime
        .load_typescript_module("loop.ts", loop_script)
        .expect("load module");

    let result: Result<(), ScriptExecutionError> =
        runtime.invoke_function("entity_bad", &loaded, "onUpdate", (0.016f32,));

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err
        .message
        .contains("exceeded maximum allowed frame duration"));
}

#[test]
fn test_error_mapping_to_correct_typescript_line() {
    let buggy_script = r#"
        // Line 2
        // Line 3
        interface Person {
            name: string;
        }
        // Line 8
        export function onUpdate(dt: number): void {
            const obj: any = null;
            obj.nonExistentMethod();
        }
    "#;

    let mut runtime = ScriptRuntime::new(SandboxConfig::default()).expect("runtime init");
    let loaded = runtime
        .load_typescript_module("buggy.ts", buggy_script)
        .expect("load module");

    let result: Result<(), ScriptExecutionError> =
        runtime.invoke_function("entity_1", &loaded, "onUpdate", (0.016f32,));

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert_eq!(err.file, Some("buggy.ts".to_string()));
    assert!(err.line.is_some());
    // Line in original source should be 11 (obj.nonExistentMethod())
    let line = err.line.unwrap();
    assert!(
        line >= 9 && line <= 13,
        "Expected line near 11, got {}",
        line
    );
}

#[test]
fn test_hot_reload_with_state_transfer() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut script_file = NamedTempFile::new().expect("create temp script");
    let script_v1 = r#"
        let count = 42;
        export function serialize(): unknown {
            return { count };
        }
        export function deserialize(state: any): void {
            count = state.count + 10;
        }
        export function getCount(): number {
            return count;
        }
    "#;
    script_file
        .write_all(script_v1.as_bytes())
        .expect("write v1");
    script_file.flush().expect("flush");

    let script_path = script_file.path().to_string_lossy().to_string();
    let mut server = ScriptServer::new(SandboxConfig::default()).expect("server init");

    // Load initial version
    let _ = server
        .get_or_load_script(&script_path)
        .expect("initial load");

    let entity = forge_core::ecs::Entity::from_raw(1);
    let active_entities = vec![(entity, "player_1".to_string(), script_path.clone())];

    // Verify initial count is 42
    {
        let loaded = server.loaded_scripts.get(&script_path).unwrap();
        let mut rt = server.runtime.0.lock().unwrap();
        let count: i32 = rt
            .invoke_function("player_1", loaded, "getCount", ())
            .expect("getCount");
        assert_eq!(count, 42);
    }

    // Sleep briefly to ensure different modification time on filesystem
    std::thread::sleep(Duration::from_millis(50));

    // Update script file to version 2 (with modified logic)
    let script_v2 = r#"
        let count = 0;
        export function serialize(): unknown {
            return { count };
        }
        export function deserialize(state: any): void {
            count = state.count + 100;
        }
        export function getCount(): number {
            return count;
        }
    "#;
    std::fs::write(&script_path, script_v2).expect("write v2");

    // Trigger reload
    let errors = server.check_and_reload_scripts(&active_entities);
    assert!(errors.is_empty(), "Reload should succeed with no errors");

    // Check count: state 42 was carried over and deserialize added 100 -> count = 142
    {
        let loaded = server.loaded_scripts.get(&script_path).unwrap();
        let mut rt = server.runtime.0.lock().unwrap();
        let count: i32 = rt
            .invoke_function("player_1", loaded, "getCount", ())
            .expect("getCount v2");
        assert_eq!(count, 142);
    }
}

#[test]
fn test_hot_reload_syntax_error_keeps_old_code_running() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut script_file = NamedTempFile::new().expect("create temp script");
    let script_v1 = r#"
        export function getSpeed(): number {
            return 150;
        }
    "#;
    script_file
        .write_all(script_v1.as_bytes())
        .expect("write v1");
    script_file.flush().expect("flush");

    let script_path = script_file.path().to_string_lossy().to_string();
    let mut server = ScriptServer::new(SandboxConfig::default()).expect("server init");

    // Initial load
    let _ = server
        .get_or_load_script(&script_path)
        .expect("initial load");

    let entity = forge_core::ecs::Entity::from_raw(2);
    let active_entities = vec![(entity, "player_2".to_string(), script_path.clone())];

    {
        let loaded = server.loaded_scripts.get(&script_path).unwrap();
        let mut rt = server.runtime.0.lock().unwrap();
        let speed: i32 = rt
            .invoke_function("player_2", loaded, "getSpeed", ())
            .expect("getSpeed v1");
        assert_eq!(speed, 150);
    }

    std::thread::sleep(Duration::from_millis(50));

    // Introduce deliberate syntax error
    let invalid_ts = r#"
        export function getSpeed() {
            return invalid syntax {{{
    "#;
    std::fs::write(&script_path, invalid_ts).expect("write invalid");

    // Reload should capture error but keep old script running
    let errors = server.check_and_reload_scripts(&active_entities);
    assert!(!errors.is_empty(), "Reload should report syntax error");

    // Previous version still exists and executes successfully
    {
        let loaded = server.loaded_scripts.get(&script_path).unwrap();
        let mut rt = server.runtime.0.lock().unwrap();
        let speed: i32 = rt
            .invoke_function("player_2", loaded, "getSpeed", ())
            .expect("getSpeed still works");
        assert_eq!(speed, 150);
    }
}
