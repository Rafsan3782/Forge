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
  PlaytestReportSchema,
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
