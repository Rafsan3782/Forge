import test from "node:test";
import assert from "node:assert/strict";
import * as fs from "node:fs/promises";
import * as path from "node:path";

import {
  ReplayLlmProvider,
  LiveRecordingLlmProvider,
} from "./cassette.ts";
import { EvalRunner } from "./runner.ts";
import {
  computeTaskWeightedScore,
  buildSuiteReport,
  compareRuns,
} from "./scoring.ts";
import { generateHtmlReport } from "./reports.ts";
import { getAllTasks } from "./tasks/index.ts";
import type { CassetteData, TaskEvalResult } from "./types.ts";

test("ReplayLlmProvider deterministically replays recorded interactions", async () => {
  const cassette: CassetteData = {
    version: 1,
    metadata: {
      recording_id: "test_rec",
      created_at: new Date().toISOString(),
      model: "fake-gemini",
    },
    interactions: [
      {
        turn_id: "turn_001",
        request: { messages: [{ role: "user", content: "hello" }] },
        response: {
          thought: "Greeting user",
          message_delta: "Hi there!",
          finish_reason: "stop",
          usage: {
            input_tokens: 10,
            output_tokens: 5,
            total_tokens: 15,
            estimated_cost_usd: 0.0001,
          },
        },
      },
    ],
  };

  const provider = new ReplayLlmProvider(cassette);
  const chunks = [];
  for await (const chunk of provider.streamTurn([{ role: "user", content: "hello" }])) {
    chunks.push(chunk);
  }

  assert.equal(chunks.length, 2);
  assert.equal(chunks[0].thought, "Greeting user");
  assert.equal(chunks[1].delta, "Hi there!");
});

test("LiveRecordingLlmProvider records turns into cassette format", async () => {
  const mockBaseLlm = {
    getModelName: () => "mock-model",
    async *streamTurn() {
      yield { thought: "thinking..." };
      yield { delta: "result", finish_reason: "stop" as const };
    },
  };

  const recorder = new LiveRecordingLlmProvider(mockBaseLlm, "test_session");
  const iterator = recorder.streamTurn([{ role: "user", content: "do task" }])[Symbol.asyncIterator]();
  while (!(await iterator.next()).done) {
    // consume stream
  }

  const cassette = recorder.getCassette();
  assert.equal(cassette.interactions.length, 1);
  assert.equal(cassette.interactions[0].response.thought, "thinking...");
  assert.equal(cassette.interactions[0].response.message_delta, "result");
});

test("Scoring calculates weighted score and reports accurately", () => {
  const mockResult: TaskEvalResult = {
    taskId: "test_task",
    taskName: "Test Task",
    passed: true,
    score: 1.0,
    checks: [
      { name: "Check 1", passed: true },
      { name: "Check 2", passed: true },
    ],
    durationMs: 500,
    turns: 2,
    tokens: 200,
    costUsd: 0.001,
    checkpointsCreated: 1,
    events: [],
    toolCalls: [],
    finalScreenshots: [],
  };

  const weighted = computeTaskWeightedScore(mockResult);
  assert.ok(weighted >= 0.9);

  const report = buildSuiteReport([mockResult], "replay");
  assert.equal(report.passedTasks, 1);
  assert.equal(report.failedTasks, 0);

  const html = generateHtmlReport(report);
  assert.ok(html.includes("Test Task"));
  assert.ok(html.includes("100%"));
});

test("compareRuns detects pass/fail, step, and cost regressions", () => {
  const baseResult: TaskEvalResult = {
    taskId: "task_a",
    taskName: "Task A",
    passed: true,
    score: 1.0,
    checks: [{ name: "Integrity", passed: true }],
    durationMs: 100,
    turns: 2,
    tokens: 100,
    costUsd: 0.001,
    checkpointsCreated: 1,
    events: [],
    toolCalls: [],
    finalScreenshots: [],
  };

  const regressedResult: TaskEvalResult = {
    ...baseResult,
    passed: false,
    turns: 8,
    costUsd: 0.08,
    checks: [{ name: "Integrity", passed: false }],
  };

  const baseReport = buildSuiteReport([baseResult], "replay");
  const candReport = buildSuiteReport([regressedResult], "replay");

  const diff = compareRuns(baseReport, candReport);
  assert.ok(diff.regressions.length >= 2);
  assert.ok(diff.regressions.some((r) => r.type === "failed_regression"));
  assert.ok(diff.regressions.some((r) => r.type === "step_regression"));
});

test("First task set (7 tasks) are properly defined and have golden solutions", () => {
  const tasks = getAllTasks();
  assert.equal(tasks.length, 7);

  const expectedIds = [
    "pong",
    "double_jump",
    "fix_script_bug",
    "patrolling_enemy",
    "tilemap_level_goal",
    "coin_counter",
    "retune_jump_floatier",
  ];

  for (const id of expectedIds) {
    const found = tasks.find((t) => t.config.id === id);
    assert.ok(found, `Expected task ${id} to exist`);
    assert.ok(found!.checks.length > 0, `Task ${id} must have checks`);
    assert.ok(typeof found!.goldenSolution === "function", `Task ${id} must have a goldenSolution`);
  }
});

test("EvalRunner executes tasks with golden solutions cleanly", async () => {
  const runner = new EvalRunner({ mode: "replay", maxConcurrency: 1, useGoldenSolution: true });

  const mockTask = {
    config: {
      id: "mock_pong",
      name: "Mock Pong",
      description: "Mock task for unit test",
      template: "hello",
      prompt: "Create entities",
      limits: { timeout_seconds: 10, max_turns: 2 },
      tags: ["test"],
    },
    taskDir: path.resolve(process.cwd(), "src/tasks/pong"),
    checks: [
      (_ctx: unknown) => ({ name: "Check Entities", passed: true, message: "OK" }),
      (_ctx: unknown) => ({ name: "Check Score", passed: true, message: "OK" }),
    ],
    goldenSolution: async (workspaceDir: string) => {
      // Golden solution creates files
      await fs.writeFile(path.join(workspaceDir, "test.txt"), "hello", "utf-8");
    },
  };

  const result = await runner.runTask(mockTask);
  assert.equal(result.taskId, "mock_pong");
  assert.equal(result.passed, true);
  assert.ok(result.score > 0);
  assert.equal(result.checks.length, 2);
});
