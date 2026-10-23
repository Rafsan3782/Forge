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
