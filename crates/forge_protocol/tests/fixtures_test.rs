use forge_protocol::*;
use std::fs;
use std::path::{Path, PathBuf};

fn get_fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/protocol/fixtures")
}

fn load_fixture(name: &str) -> String {
    let path = get_fixtures_dir().join(name);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("Failed to read fixture {}: {}", path.display(), err))
}

fn roundtrip<T>(fixture_name: &str) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + PartialEq,
{
    let raw = load_fixture(fixture_name);
    let deserialized: T = serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("Failed to deserialize {}: {}", fixture_name, err));

    let reserialized = serde_json::to_string(&deserialized)
        .unwrap_or_else(|err| panic!("Failed to reserialize {}: {}", fixture_name, err));

    let roundtripped: T = serde_json::from_str(&reserialized).unwrap_or_else(|err| {
        panic!(
            "Failed to deserialize reserialized {}: {}",
            fixture_name, err
        )
    });

    assert_eq!(
        deserialized, roundtripped,
        "Mismatch in roundtrip for {}",
        fixture_name
    );
    deserialized
}

#[test]
fn test_envelopes() {
    let req: JsonRpcRequest = roundtrip("envelope_request.json");
    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.method, "engine.ping");

    let res: JsonRpcResponse = roundtrip("envelope_response.json");
    assert_eq!(res.jsonrpc, "2.0");

    let err: JsonRpcError = roundtrip("envelope_error.json");
    assert_eq!(err.jsonrpc, "2.0");
    assert_eq!(err.error.code, -32600);

    let notif: JsonRpcNotification = roundtrip("envelope_notification.json");
    assert_eq!(notif.jsonrpc, "2.0");
    assert_eq!(notif.method, "engine.heartbeat");
}

#[test]
fn test_engine_ping() {
    let params: EnginePingParams = roundtrip("engine_ping_params.json");
    assert_eq!(params.timestamp, 1727990400000.0);

    let result: EnginePingResult = roundtrip("engine_ping_result.json");
    assert_eq!(result.status, "ok");
    assert_eq!(result.timestamp, 1727990400000.0);
}

#[test]
fn test_scene_get_tree() {
    let params: SceneGetTreeParams = roundtrip("scene_get_tree_params.json");
    assert_eq!(params.root_entity_id, Some("root-001".to_string()));

    let result: SceneGetTreeResult = roundtrip("scene_get_tree_result.json");
    assert_eq!(result.root_entities.len(), 2);
    assert_eq!(result.root_entities[0].name, "MainCamera");
    assert_eq!(result.root_entities[1].children.len(), 1);
}

#[test]
fn test_scene_create_entity() {
    let params: SceneCreateEntityParams = roundtrip("scene_create_entity_params.json");
    assert_eq!(params.name, "LightSource");
    assert_eq!(params.parent_id, Some("entity-002".to_string()));
    assert!(params.components.is_some());

    let result: SceneCreateEntityResult = roundtrip("scene_create_entity_result.json");
    assert_eq!(result.entity_id, "entity-004");
}

#[test]
fn test_scene_set_component() {
    let params: SceneSetComponentParams = roundtrip("scene_set_component_params.json");
    assert_eq!(params.entity_id, "entity-002");
    assert_eq!(params.component_name, "Transform");

    let result: SceneSetComponentResult = roundtrip("scene_set_component_result.json");
    assert!(result.success);
}

#[test]
fn test_scene_delete_entity() {
    let params: SceneDeleteEntityParams = roundtrip("scene_delete_entity_params.json");
    assert_eq!(params.entity_id, "entity-004");

    let result: SceneDeleteEntityResult = roundtrip("scene_delete_entity_result.json");
    assert!(result.success);
}

#[test]
fn test_project_run_and_stop() {
    let run_params: ProjectRunParams = roundtrip("project_run_params.json");
    assert_eq!(run_params.project_path, "C:/Projects/SampleGame");
    assert_eq!(run_params.mode, "debug");

    let run_result: ProjectRunResult = roundtrip("project_run_result.json");
    assert_eq!(run_result.session_id, "sess-9876");
    assert_eq!(run_result.process_id, 4321);

    let stop_params: ProjectStopParams = roundtrip("project_stop_params.json");
    assert_eq!(stop_params.session_id, "sess-9876");

    let stop_result: ProjectStopResult = roundtrip("project_stop_result.json");
    assert!(stop_result.stopped);
}

#[test]
fn test_game_screenshot() {
    let params: GameScreenshotParams = roundtrip("game_screenshot_params.json");
    assert_eq!(params.format, "png");
    assert_eq!(params.width, Some(1920));
    assert_eq!(params.height, Some(1080));

    let result: GameScreenshotResult = roundtrip("game_screenshot_result.json");
    assert_eq!(result.width, 1920);
    assert_eq!(result.height, 1080);
    assert_eq!(result.format, "png");
}

#[test]
fn test_playtest_report() {
    let report: PlaytestReport = roundtrip("playtest_report.json");
    assert_eq!(report.build_id, "build-v0.1.0-alpha");
    assert_eq!(report.checkpoint_id, "cp-level-01-boss");
    assert_eq!(report.errors.len(), 1);
    assert_eq!(report.logs.len(), 2);
    assert_eq!(report.screenshots.len(), 2);
    assert_eq!(report.input_trace.len(), 2);
    assert_eq!(report.perf_stats.fps_avg, 59.4);
    assert_eq!(report.user_notes.len(), 1);
    assert_eq!(report.severity, PlaytestSeverity::Medium);
}

#[test]
fn test_script_tools_fixtures() {
    let check_params: ScriptCheckParams = roundtrip("script_check_params.json");
    assert_eq!(
        check_params.project_path,
        Some("templates/script-demo".to_string())
    );

    let check_result: ScriptCheckResult = roundtrip("script_check_result.json");
    assert!(check_result.success);
    assert_eq!(check_result.diagnostics.len(), 1);
    assert_eq!(check_result.diagnostics[0].code, "TS2322");

    let list_params: ScriptListParams = roundtrip("script_list_params.json");
    assert_eq!(
        list_params.project_path,
        Some("templates/script-demo".to_string())
    );

    let list_result: ScriptListResult = roundtrip("script_list_result.json");
    assert_eq!(list_result.scripts.len(), 2);

    let read_params: ScriptReadParams = roundtrip("script_read_params.json");
    assert_eq!(read_params.path, "scripts/player.ts");

    let read_result: ScriptReadResult = roundtrip("script_read_result.json");
    assert_eq!(read_result.path, "scripts/player.ts");

    let write_params: ScriptWriteParams = roundtrip("script_write_params.json");
    assert_eq!(write_params.path, "scripts/player.ts");

    let write_result: ScriptWriteResult = roundtrip("script_write_result.json");
    assert!(write_result.success);
}

#[test]
fn test_edit_ops_fixtures() {
    let _undo_params: EditUndoParams = roundtrip("edit_undo_params.json");
    let undo_result: EditUndoResult = roundtrip("edit_undo_result.json");
    assert!(undo_result.success);
    assert_eq!(
        undo_result.undone_command.as_deref(),
        Some("Set Transform2D on player")
    );

    let history_result: EditGetHistoryResult = roundtrip("edit_get_history_result.json");
    assert_eq!(history_result.undo_stack.len(), 1);
    assert_eq!(history_result.undo_stack[0].author, "agent");

    let schemas_result: SceneGetComponentSchemasResult =
        roundtrip("scene_get_component_schemas_result.json");
    assert_eq!(schemas_result.schemas.len(), 1);
    assert_eq!(schemas_result.schemas[0].name, "Transform2D");

    let dirty_result: SceneGetDirtyResult = roundtrip("scene_get_dirty_result.json");
    assert!(dirty_result.is_dirty);

    let save_result: SceneSaveResult = roundtrip("scene_save_result.json");
    assert!(save_result.success);

    let play_start: PlayStartResult = roundtrip("play_start_result.json");
    assert_eq!(play_start.state, "playing");

    let play_stop: PlayStopResult = roundtrip("play_stop_result.json");
    assert_eq!(play_stop.state, "stopped");
}
