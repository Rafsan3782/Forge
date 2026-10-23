import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  JsonRpcRequestSchema,
  JsonRpcResponseSchema,
  JsonRpcErrorSchema,
  JsonRpcNotificationSchema,
  EnginePingParamsSchema,
  EnginePingResultSchema,
  SceneGetTreeParamsSchema,
  SceneGetTreeResultSchema,
  SceneCreateEntityParamsSchema,
  SceneCreateEntityResultSchema,
  SceneSetComponentParamsSchema,
  SceneSetComponentResultSchema,
  SceneDeleteEntityParamsSchema,
  SceneDeleteEntityResultSchema,
  ProjectRunParamsSchema,
  ProjectRunResultSchema,
  ProjectStopParamsSchema,
  ProjectStopResultSchema,
  GameScreenshotParamsSchema,
  GameScreenshotResultSchema,
  GameSimulateInputParamsSchema,
  GameSimulateInputResultSchema,
  AssetListParamsSchema,
  AssetListResultSchema,
  AssetGetInfoParamsSchema,
  AssetGetInfoResultSchema,
  AssetImportParamsSchema,
  AssetImportResultSchema,
  PhysicsRaycastParamsSchema,
  PhysicsRaycastResultSchema,
  PlaytestReportSchema,
  TilemapGetParamsSchema,
  TilemapGetResultSchema,
  TilemapSetTileParamsSchema,
  TilemapSetTileResultSchema,
  TilemapFillRectParamsSchema,
  TilemapFillRectResultSchema,
  TilemapClearParamsSchema,
  TilemapClearResultSchema,
  AudioPlayParamsSchema,
  AudioPlayResultSchema,
  AudioStopParamsSchema,
  AudioStopResultSchema,
  AudioSetBusVolumeParamsSchema,
  AudioSetBusVolumeResultSchema,
  ScriptErrorEventSchema,
} from "./generated/index.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const FIXTURES_DIR = path.resolve(__dirname, "../fixtures");

function loadFixture(filename: string): unknown {
  const filePath = path.join(FIXTURES_DIR, filename);
  const content = fs.readFileSync(filePath, "utf-8").replace(/^\uFEFF/, "");
  return JSON.parse(content);
}

test("JSON-RPC envelopes serialize and deserialize against fixtures", () => {
  const reqData = loadFixture("envelope_request.json");
  const parsedReq = JsonRpcRequestSchema.parse(reqData);
  const reserializedReq = JSON.parse(JSON.stringify(parsedReq));
  assert.deepStrictEqual(JsonRpcRequestSchema.parse(reserializedReq), parsedReq);

  const resData = loadFixture("envelope_response.json");
  const parsedRes = JsonRpcResponseSchema.parse(resData);
  const reserializedRes = JSON.parse(JSON.stringify(parsedRes));
  assert.deepStrictEqual(JsonRpcResponseSchema.parse(reserializedRes), parsedRes);

  const errData = loadFixture("envelope_error.json");
  const parsedErr = JsonRpcErrorSchema.parse(errData);
  const reserializedErr = JSON.parse(JSON.stringify(parsedErr));
  assert.deepStrictEqual(JsonRpcErrorSchema.parse(reserializedErr), parsedErr);

  const notifData = loadFixture("envelope_notification.json");
  const parsedNotif = JsonRpcNotificationSchema.parse(notifData);
  const reserializedNotif = JSON.parse(JSON.stringify(parsedNotif));
  assert.deepStrictEqual(JsonRpcNotificationSchema.parse(reserializedNotif), parsedNotif);
});

test("engine.ping serialize and deserialize against fixtures", () => {
  const paramsData = loadFixture("engine_ping_params.json");
  const parsedParams = EnginePingParamsSchema.parse(paramsData);
  const reserializedParams = JSON.parse(JSON.stringify(parsedParams));
  assert.deepStrictEqual(EnginePingParamsSchema.parse(reserializedParams), parsedParams);

  const resultData = loadFixture("engine_ping_result.json");
  const parsedResult = EnginePingResultSchema.parse(resultData);
  const reserializedResult = JSON.parse(JSON.stringify(parsedResult));
  assert.deepStrictEqual(EnginePingResultSchema.parse(reserializedResult), parsedResult);
});

test("scene operations serialize and deserialize against fixtures", () => {
  // scene.get_tree
  const getTreeParams = loadFixture("scene_get_tree_params.json");
  const parsedGetTreeParams = SceneGetTreeParamsSchema.parse(getTreeParams);
  assert.deepStrictEqual(
    SceneGetTreeParamsSchema.parse(JSON.parse(JSON.stringify(parsedGetTreeParams))),
    parsedGetTreeParams,
  );

  const getTreeResult = loadFixture("scene_get_tree_result.json");
  const parsedGetTreeResult = SceneGetTreeResultSchema.parse(getTreeResult);
  assert.deepStrictEqual(
    SceneGetTreeResultSchema.parse(JSON.parse(JSON.stringify(parsedGetTreeResult))),
    parsedGetTreeResult,
  );

  // scene.create_entity
  const createEntityParams = loadFixture("scene_create_entity_params.json");
  const parsedCreateEntityParams = SceneCreateEntityParamsSchema.parse(createEntityParams);
  assert.deepStrictEqual(
    SceneCreateEntityParamsSchema.parse(JSON.parse(JSON.stringify(parsedCreateEntityParams))),
    parsedCreateEntityParams,
  );

  const createEntityResult = loadFixture("scene_create_entity_result.json");
  const parsedCreateEntityResult = SceneCreateEntityResultSchema.parse(createEntityResult);
  assert.deepStrictEqual(
    SceneCreateEntityResultSchema.parse(JSON.parse(JSON.stringify(parsedCreateEntityResult))),
    parsedCreateEntityResult,
  );

  // scene.set_component
  const setComponentParams = loadFixture("scene_set_component_params.json");
  const parsedSetComponentParams = SceneSetComponentParamsSchema.parse(setComponentParams);
  assert.deepStrictEqual(
    SceneSetComponentParamsSchema.parse(JSON.parse(JSON.stringify(parsedSetComponentParams))),
    parsedSetComponentParams,
  );

  const setComponentResult = loadFixture("scene_set_component_result.json");
  const parsedSetComponentResult = SceneSetComponentResultSchema.parse(setComponentResult);
  assert.deepStrictEqual(
    SceneSetComponentResultSchema.parse(JSON.parse(JSON.stringify(parsedSetComponentResult))),
    parsedSetComponentResult,
  );

  // scene.delete_entity
  const deleteEntityParams = loadFixture("scene_delete_entity_params.json");
  const parsedDeleteEntityParams = SceneDeleteEntityParamsSchema.parse(deleteEntityParams);
  assert.deepStrictEqual(
    SceneDeleteEntityParamsSchema.parse(JSON.parse(JSON.stringify(parsedDeleteEntityParams))),
    parsedDeleteEntityParams,
  );

  const deleteEntityResult = loadFixture("scene_delete_entity_result.json");
  const parsedDeleteEntityResult = SceneDeleteEntityResultSchema.parse(deleteEntityResult);
  assert.deepStrictEqual(
    SceneDeleteEntityResultSchema.parse(JSON.parse(JSON.stringify(parsedDeleteEntityResult))),
    parsedDeleteEntityResult,
  );
});

test("project operations serialize and deserialize against fixtures", () => {
  const runParams = loadFixture("project_run_params.json");
  const parsedRunParams = ProjectRunParamsSchema.parse(runParams);
  assert.deepStrictEqual(
    ProjectRunParamsSchema.parse(JSON.parse(JSON.stringify(parsedRunParams))),
    parsedRunParams,
  );

  const runResult = loadFixture("project_run_result.json");
  const parsedRunResult = ProjectRunResultSchema.parse(runResult);
  assert.deepStrictEqual(
    ProjectRunResultSchema.parse(JSON.parse(JSON.stringify(parsedRunResult))),
    parsedRunResult,
  );

  const stopParams = loadFixture("project_stop_params.json");
  const parsedStopParams = ProjectStopParamsSchema.parse(stopParams);
  assert.deepStrictEqual(
    ProjectStopParamsSchema.parse(JSON.parse(JSON.stringify(parsedStopParams))),
    parsedStopParams,
  );

  const stopResult = loadFixture("project_stop_result.json");
  const parsedStopResult = ProjectStopResultSchema.parse(stopResult);
  assert.deepStrictEqual(
    ProjectStopResultSchema.parse(JSON.parse(JSON.stringify(parsedStopResult))),
    parsedStopResult,
  );
});

test("game.screenshot serialize and deserialize against fixtures", () => {
  const params = loadFixture("game_screenshot_params.json");
  const parsedParams = GameScreenshotParamsSchema.parse(params);
  assert.deepStrictEqual(
    GameScreenshotParamsSchema.parse(JSON.parse(JSON.stringify(parsedParams))),
    parsedParams,
  );

  const result = loadFixture("game_screenshot_result.json");
  const parsedResult = GameScreenshotResultSchema.parse(result);
  assert.deepStrictEqual(
    GameScreenshotResultSchema.parse(JSON.parse(JSON.stringify(parsedResult))),
    parsedResult,
  );
});

test("game.simulate_input serialize and deserialize against fixtures", () => {
  const params = loadFixture("game_simulate_input_params.json");
  const parsedParams = GameSimulateInputParamsSchema.parse(params);
  assert.deepStrictEqual(
    GameSimulateInputParamsSchema.parse(JSON.parse(JSON.stringify(parsedParams))),
    parsedParams,
  );

  const result = loadFixture("game_simulate_input_result.json");
  const parsedResult = GameSimulateInputResultSchema.parse(result);
  assert.deepStrictEqual(
    GameSimulateInputResultSchema.parse(JSON.parse(JSON.stringify(parsedResult))),
    parsedResult,
  );
});

test("asset operations serialize and deserialize against fixtures", () => {
  // asset.list
  const listParams = loadFixture("asset_list_params.json");
  const parsedListParams = AssetListParamsSchema.parse(listParams);
  assert.deepStrictEqual(
    AssetListParamsSchema.parse(JSON.parse(JSON.stringify(parsedListParams))),
    parsedListParams,
  );

  const listResult = loadFixture("asset_list_result.json");
  const parsedListResult = AssetListResultSchema.parse(listResult);
  assert.deepStrictEqual(
    AssetListResultSchema.parse(JSON.parse(JSON.stringify(parsedListResult))),
    parsedListResult,
  );

  // asset.get_info
  const getInfoParams = loadFixture("asset_get_info_params.json");
  const parsedGetInfoParams = AssetGetInfoParamsSchema.parse(getInfoParams);
  assert.deepStrictEqual(
    AssetGetInfoParamsSchema.parse(JSON.parse(JSON.stringify(parsedGetInfoParams))),
    parsedGetInfoParams,
  );

  const getInfoResult = loadFixture("asset_get_info_result.json");
  const parsedGetInfoResult = AssetGetInfoResultSchema.parse(getInfoResult);
  assert.deepStrictEqual(
    AssetGetInfoResultSchema.parse(JSON.parse(JSON.stringify(parsedGetInfoResult))),
    parsedGetInfoResult,
  );

  // asset.import
  const importParams = loadFixture("asset_import_params.json");
  const parsedImportParams = AssetImportParamsSchema.parse(importParams);
  assert.deepStrictEqual(
    AssetImportParamsSchema.parse(JSON.parse(JSON.stringify(parsedImportParams))),
    parsedImportParams,
  );

  const importResult = loadFixture("asset_import_result.json");
  const parsedImportResult = AssetImportResultSchema.parse(importResult);
  assert.deepStrictEqual(
    AssetImportResultSchema.parse(JSON.parse(JSON.stringify(parsedImportResult))),
    parsedImportResult,
  );
});

test("physics.raycast serialize and deserialize against fixtures", () => {
  const raycastParams = loadFixture("physics_raycast_params.json");
  const parsedRaycastParams = PhysicsRaycastParamsSchema.parse(raycastParams);
  assert.deepStrictEqual(
    PhysicsRaycastParamsSchema.parse(JSON.parse(JSON.stringify(parsedRaycastParams))),
    parsedRaycastParams,
  );

  const raycastResult = loadFixture("physics_raycast_result.json");
  const parsedRaycastResult = PhysicsRaycastResultSchema.parse(raycastResult);
  assert.deepStrictEqual(
    PhysicsRaycastResultSchema.parse(JSON.parse(JSON.stringify(parsedRaycastResult))),
    parsedRaycastResult,
  );
  assert.strictEqual(parsedRaycastResult.hit, true);
  assert.strictEqual(parsedRaycastResult.data?.entity_id, "floor_entity");
});

test("playtest.report serialize and deserialize against fixtures", () => {
  const reportData = loadFixture("playtest_report.json");
  const parsedReport = PlaytestReportSchema.parse(reportData);
  assert.deepStrictEqual(
    PlaytestReportSchema.parse(JSON.parse(JSON.stringify(parsedReport))),
    parsedReport,
  );
  assert.strictEqual(parsedReport.build_id, "build-v0.1.0-alpha");
  assert.strictEqual(parsedReport.checkpoint_id, "cp-level-01-boss");
  assert.strictEqual(parsedReport.errors.length, 1);
  assert.strictEqual(parsedReport.logs.length, 2);
  assert.strictEqual(parsedReport.screenshots.length, 2);
  assert.strictEqual(parsedReport.input_trace.length, 2);
  assert.strictEqual(parsedReport.severity, "medium");
});

test("tilemap operations serialize and deserialize against fixtures", () => {
  const getParams = loadFixture("tilemap_get_params.json");
  const parsedGetParams = TilemapGetParamsSchema.parse(getParams);
  assert.deepStrictEqual(
    TilemapGetParamsSchema.parse(JSON.parse(JSON.stringify(parsedGetParams))),
    parsedGetParams,
  );

  const getResult = loadFixture("tilemap_get_result.json");
  const parsedGetResult = TilemapGetResultSchema.parse(getResult);
  assert.deepStrictEqual(
    TilemapGetResultSchema.parse(JSON.parse(JSON.stringify(parsedGetResult))),
    parsedGetResult,
  );

  const setParams = loadFixture("tilemap_set_tile_params.json");
  const parsedSetParams = TilemapSetTileParamsSchema.parse(setParams);
  assert.deepStrictEqual(
    TilemapSetTileParamsSchema.parse(JSON.parse(JSON.stringify(parsedSetParams))),
    parsedSetParams,
  );

  const setResult = loadFixture("tilemap_set_tile_result.json");
  const parsedSetResult = TilemapSetTileResultSchema.parse(setResult);
  assert.deepStrictEqual(
    TilemapSetTileResultSchema.parse(JSON.parse(JSON.stringify(parsedSetResult))),
    parsedSetResult,
  );

  const fillParams = loadFixture("tilemap_fill_rect_params.json");
  const parsedFillParams = TilemapFillRectParamsSchema.parse(fillParams);
  assert.deepStrictEqual(
    TilemapFillRectParamsSchema.parse(JSON.parse(JSON.stringify(parsedFillParams))),
    parsedFillParams,
  );

  const fillResult = loadFixture("tilemap_fill_rect_result.json");
  const parsedFillResult = TilemapFillRectResultSchema.parse(fillResult);
  assert.deepStrictEqual(
    TilemapFillRectResultSchema.parse(JSON.parse(JSON.stringify(parsedFillResult))),
    parsedFillResult,
  );

  const clearParams = loadFixture("tilemap_clear_params.json");
  const parsedClearParams = TilemapClearParamsSchema.parse(clearParams);
  assert.deepStrictEqual(
    TilemapClearParamsSchema.parse(JSON.parse(JSON.stringify(parsedClearParams))),
    parsedClearParams,
  );

  const clearResult = loadFixture("tilemap_clear_result.json");
  const parsedClearResult = TilemapClearResultSchema.parse(clearResult);
  assert.deepStrictEqual(
    TilemapClearResultSchema.parse(JSON.parse(JSON.stringify(parsedClearResult))),
    parsedClearResult,
  );
});

test("audio operations serialize and deserialize against fixtures", () => {
  const playParams = loadFixture("audio_play_params.json");
  const parsedPlayParams = AudioPlayParamsSchema.parse(playParams);
  assert.deepStrictEqual(
    AudioPlayParamsSchema.parse(JSON.parse(JSON.stringify(parsedPlayParams))),
    parsedPlayParams,
  );

  const playResult = loadFixture("audio_play_result.json");
  const parsedPlayResult = AudioPlayResultSchema.parse(playResult);
  assert.deepStrictEqual(
    AudioPlayResultSchema.parse(JSON.parse(JSON.stringify(parsedPlayResult))),
    parsedPlayResult,
  );

  const stopParams = loadFixture("audio_stop_params.json");
  const parsedStopParams = AudioStopParamsSchema.parse(stopParams);
  assert.deepStrictEqual(
    AudioStopParamsSchema.parse(JSON.parse(JSON.stringify(parsedStopParams))),
    parsedStopParams,
  );

  const stopResult = loadFixture("audio_stop_result.json");
  const parsedStopResult = AudioStopResultSchema.parse(stopResult);
  assert.deepStrictEqual(
    AudioStopResultSchema.parse(JSON.parse(JSON.stringify(parsedStopResult))),
    parsedStopResult,
  );

  const volParams = loadFixture("audio_set_bus_volume_params.json");
  const parsedVolParams = AudioSetBusVolumeParamsSchema.parse(volParams);
  assert.deepStrictEqual(
    AudioSetBusVolumeParamsSchema.parse(JSON.parse(JSON.stringify(parsedVolParams))),
    parsedVolParams,
  );

  const volResult = loadFixture("audio_set_bus_volume_result.json");
  const parsedVolResult = AudioSetBusVolumeResultSchema.parse(volResult);
  assert.deepStrictEqual(
    AudioSetBusVolumeResultSchema.parse(JSON.parse(JSON.stringify(parsedVolResult))),
    parsedVolResult,
  );
});

test("script operations and errors serialize and deserialize", () => {
  const errorEvent = {
    entity_id: "player_1",
    script_path: "scripts/player.ts",
    message: "Cannot read property of undefined",
    stack: "TypeError: Cannot read property of undefined\n    at onUpdate (scripts/player.ts:15:5)",
    file: "scripts/player.ts",
    line: 15,
    column: 5,
  };
  const parsed = ScriptErrorEventSchema.parse(errorEvent);
  assert.strictEqual(parsed.line, 15);
  assert.strictEqual(parsed.column, 5);
  assert.deepStrictEqual(ScriptErrorEventSchema.parse(JSON.parse(JSON.stringify(parsed))), parsed);
});
