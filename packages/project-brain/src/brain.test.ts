import { test } from "node:test";
import * as assert from "node:assert/strict";
import * as fs from "node:fs/promises";
import * as path from "node:path";
import * as os from "node:os";
import {
  parseSections,
  patchSection,
  mergeSectionsThreeWay,
  compactMemory,
  buildContextPack,
  ProjectBrain,
  BrainToolProvider,
} from "./index.ts";

test("Markdown section parser and patcher keeps other sections", () => {
  const original = `# Platformer Game

## Concept
<!-- id: concept -->
A fast-paced retro 2D platformer.

## Pillars
<!-- id: pillars -->
- Precision jump mechanics
- Rewarding exploration

## Core Loop
<!-- id: core-loop -->
Run, jump, collect coins, avoid hazards, reach exit flag.
`;

  const patched = patchSection(
    original,
    "core-loop",
    "Run, jump, wall-slide, collect coins, avoid spikes, reach exit flag."
  );

  assert.ok(patched.includes("A fast-paced retro 2D platformer."));
  assert.ok(patched.includes("Precision jump mechanics"));
  assert.ok(patched.includes("wall-slide"));

  const sections = parseSections(patched);
  assert.equal(sections.length, 4);
});

test("Three-way merge at section level preserves concurrent user edits", () => {
  const base = `## Concept\nA retro platformer.\n\n## Controls\nArrows to move, Space to jump.`;
  const user = `## Concept\nA retro platformer.\n\n## Controls\nWASD or Arrows to move, Space to jump.`;
  const agent = `## Concept\nA cyberpunk retro platformer.\n\n## Controls\nArrows to move, Space to jump.`;

  const merge = mergeSectionsThreeWay(base, user, agent);
  assert.equal(merge.conflicts.length, 0, "Non-overlapping edits should merge cleanly");
  assert.ok(merge.mergedContent.includes("cyberpunk retro platformer"));
  assert.ok(merge.mergedContent.includes("WASD or Arrows"));
});

test("Three-way merge detects conflicting edits at section level", () => {
  const base = `## Controls\nSpace to jump.`;
  const user = `## Controls\nSpace or W to jump.`;
  const agent = `## Controls\nUp arrow to jump.`;

  const merge = mergeSectionsThreeWay(base, user, agent);
  assert.equal(merge.conflicts.length, 1);
  assert.equal(merge.conflicts[0].sectionId, "controls");
  assert.ok(merge.mergedContent.includes("<<<<<<< USER"));
  assert.ok(merge.mergedContent.includes(">>>>>>> AGENT"));
});

test("Context pack stays strictly within token budget", () => {
  const gameMd = `# Platformer\n` + "Concept: Super long description...\n".repeat(200);
  const tasksMd = `- [ ] TASK-001: Task one\n- [ ] TASK-002: Task two\n`.repeat(50);
  const decisionsMd = `### 2026-10-26: Tech choice\n- **Decision**: Rust\n- **Why**: Speed\n`.repeat(20);
  const mapJson = {
    scenes: ["scene.json"],
    entities: { p1: { name: "Player" }, c1: { name: "Coin" } },
    scripts: [],
    assets: [],
    input_actions: ["jump", "move"],
  };

  const budget = 500;
  const pack = buildContextPack(gameMd, tasksMd, decisionsMd, mapJson, "", {
    tokenBudget: budget,
  });

  assert.ok(
    pack.totalEstimatedTokens <= budget,
    `Total tokens ${pack.totalEstimatedTokens} must stay within budget ${budget}`
  );
  assert.ok(pack.gameSummary.length > 0);
  assert.ok(pack.mapOverview !== null);
});

test("Memory compaction summarizes older notes when cap exceeded", () => {
  let memory = "# Agent Memory\n\n## Active Notes\n";
  for (let i = 1; i <= 30; i++) {
    memory += `- [2026-10-26 12:${String(i).padStart(2, "0")}]: Note item ${i} with information about platformer.\n`;
  }

  const compacted = compactMemory(memory, 500);
  assert.ok(compacted.includes("<!-- ARCHIVE_SUMMARY_START -->"));
  assert.ok(compacted.includes("Compacted Knowledge Summary"));
  assert.ok(compacted.includes("## Active Notes"));
  assert.ok(Buffer.byteLength(compacted, "utf8") < Buffer.byteLength(memory, "utf8"));
});

test("ProjectBrain class satisfies prompt-18 contract and operates file suite", async () => {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "forge-brain-test-"));

  try {
    const brain = new ProjectBrain(tempDir);

    await brain.updateSection("GAME.md", "concept", "A fast 2D platformer.");
    await brain.addTask({ title: "Implement double jump", status: "todo", acceptance_checks: ["Two jumps possible"] });
    await brain.addDecision("Use ECS", "Performance and clean separation");
    await brain.remember("Player jump velocity tuned to 440");

    await brain.recordKnowledge("speed", "220 px/s");
    const query = await brain.queryKnowledge("speed");
    assert.ok(query.length >= 1);
    assert.ok(query[0].includes("220 px/s"));

    const snapshot = await brain.getContextSnapshot();
    assert.ok(snapshot.current_tasks);

    const tools = new BrainToolProvider(brain);
    const toolList = await tools.listTools();
    assert.equal(toolList.length, 6);

    const readRes = await tools.executeTool("brain.read", { token_budget: 2000 });
    assert.equal(readRes.is_error, false);
    assert.ok((readRes.content as any).gameSummary);

    const updateRes = await tools.executeTool("brain.update_task", {
      task_id: "TASK-001",
      status: "in_progress",
    });
    assert.equal(updateRes.is_error, false);
    assert.equal((updateRes.content as any).status, "in_progress");
  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
});
