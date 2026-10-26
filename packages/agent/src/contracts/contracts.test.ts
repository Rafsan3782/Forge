import test from "node:test";
import assert from "node:assert/strict";

import {
  FakeLlmProvider,
  FakeToolProvider,
  FakeSafetyPolicy,
  FakeCheckpointManager,
  FakeOrchestrator,
  FakeSkillProvider,
  FakeProjectBrain,
  FakeDocsProvider,
} from "./fakes.ts";

import type { AgentSettings, AgentEvent } from "@forge/protocol";

const defaultSettings: AgentSettings = {
  permission_mode: "ask_risky",
  model_roles: {
    primary: "gemini-2.5-pro",
    fast: "gemini-2.5-flash",
  },
  budgets: {
    max_tokens: 100000,
    max_cost_usd: 1.0,
    max_turns: 25,
    usage: {
      input_tokens: 0,
      output_tokens: 0,
      total_tokens: 0,
      estimated_cost_usd: 0,
    },
  },
};

test("FakeLlmProvider streams deterministic turns and captures call history", async () => {
  const llm = new FakeLlmProvider();
  llm.enqueueTurn([
    { thought: "Thinking...", delta: "Hello" },
    { delta: " world!", finish_reason: "stop" },
  ]);

  const chunks: string[] = [];
  for await (const chunk of llm.streamTurn([{ role: "user", content: "Hi" }])) {
    if (chunk.delta) chunks.push(chunk.delta);
  }

  assert.equal(chunks.join(""), "Hello world!");
  assert.equal(llm.getRecordedCalls().length, 1);
  assert.equal(llm.getRecordedCalls()[0].messages[0].content, "Hi");
});

test("FakeToolProvider executes tools and logs invocations", async () => {
  const tools = new FakeToolProvider();
  const list = await tools.listTools();
  assert.ok(list.length >= 2);
  assert.ok(list.some((t) => t.name === "scene.get_tree"));

  const res = await tools.executeTool("script.write", {
    path: "scripts/test.ts",
    content: "export const x = 1;",
  });

  assert.equal(res.is_error, false);
  assert.equal(tools.getInvocations().length, 1);
  assert.equal(tools.getInvocations()[0].name, "script.write");
});

test("FakeSafetyPolicy evaluates permissions based on permission_mode", async () => {
  const safety = new FakeSafetyPolicy();
  const toolList = await new FakeToolProvider().listTools();
  const riskyTool = toolList.find((t) => t.name === "script.write")!;
  const safeTool = toolList.find((t) => t.name === "scene.get_tree")!;

  // Autonomous mode allows without permission
  const autoEval = await safety.evaluate(riskyTool, {}, { ...defaultSettings, permission_mode: "autonomous" });
  assert.equal(autoEval.requires_permission, false);

  // Ask risky mode requires permission for riskyTool
  const riskyEval = await safety.evaluate(riskyTool, {}, defaultSettings);
  assert.equal(riskyEval.requires_permission, true);

  // Ask risky mode does not require permission for safeTool
  const safeEval = await safety.evaluate(safeTool, {}, defaultSettings);
  assert.equal(safeEval.requires_permission, false);
});

test("FakeCheckpointManager creates and lists checkpoints", async () => {
  const checkpoints = new FakeCheckpointManager();
  const cp1 = await checkpoints.createCheckpoint("run_1", "Initial setup", ["scene.json"]);

  assert.ok(cp1.id.startsWith("chk_"));
  assert.equal(cp1.run_id, "run_1");

  const list = await checkpoints.listCheckpoints("run_1");
  assert.equal(list.length, 1);
  assert.equal(list[0].id, cp1.id);

  const canRevert = await checkpoints.revertToCheckpoint(cp1.id);
  assert.equal(canRevert, true);
});

test("FakeOrchestrator runs full lifecycle and emits AgentEvents", async () => {
  const llm = new FakeLlmProvider();
  llm.enqueueTurn([
    {
      thought: "Inspecting scene",
      tool_calls: [
        { id: "call_1", name: "scene.get_tree", arguments: {} },
      ],
      delta: "Inspected scene.",
      finish_reason: "tool_calls",
    },
  ]);

  const tools = new FakeToolProvider();
  const safety = new FakeSafetyPolicy();
  const checkpoints = new FakeCheckpointManager();

  const orchestrator = new FakeOrchestrator(llm, tools, safety, checkpoints, defaultSettings);

  const events: AgentEvent[] = [];
  orchestrator.subscribeEvents((evt) => events.push(evt));

  const runId = await orchestrator.startRun("Add a coin sprite");
  assert.ok(runId);

  const eventTypes = events.map((e) => e.type);
  assert.ok(eventTypes.includes("run_started"));
  assert.ok(eventTypes.includes("checkpoint_created"));
  assert.ok(eventTypes.includes("plan_updated"));
  assert.ok(eventTypes.includes("tool_call_started"));
  assert.ok(eventTypes.includes("tool_call_finished"));
  assert.ok(eventTypes.includes("message_done"));
  assert.ok(eventTypes.includes("run_finished"));

  const summary = await orchestrator.getSummary(runId);
  assert.ok(summary);
  assert.equal(summary?.status, "completed");

  const plan = await orchestrator.getCurrentPlan();
  assert.ok(plan);
  assert.equal(plan?.steps.length, 1);
});

test("FakeSkillProvider, FakeProjectBrain, and FakeDocsProvider contracts", async () => {
  const skills = new FakeSkillProvider();
  const skillList = await skills.listSkills();
  assert.ok(skillList.length > 0);
  const skill = await skills.getSkill("platformer-builder");
  assert.ok(skill);

  const brain = new FakeProjectBrain();
  await brain.recordKnowledge("player_speed", "120 px/s");
  const query = await brain.queryKnowledge("speed");
  assert.equal(query.length, 1);
  assert.ok(query[0].includes("120 px/s"));

  const docs = new FakeDocsProvider();
  const doc = await docs.getDoc("docs/ENGINE_STATE.md");
  assert.ok(doc && doc.includes("Forge Engine State Audit"));
  const search = await docs.searchDocs("cassette");
  assert.equal(search.length, 1);
});
