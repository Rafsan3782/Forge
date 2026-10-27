import test from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { DocsToolProvider } from "./provider.ts";
import { EVALUATION_QUESTIONS } from "./eval_data.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const docsRoot = path.resolve(__dirname, "../../../docs/src");

test("DocsToolProvider lists standard tools conforming to ToolDescriptor schema", async () => {
  const provider = new DocsToolProvider(docsRoot);
  const tools = await provider.listTools();

  assert.equal(tools.length, 5);
  const toolNames = tools.map((t) => t.name);
  assert.ok(toolNames.includes("docs.search"));
  assert.ok(toolNames.includes("docs.read"));
  assert.ok(toolNames.includes("docs.list_recipes"));
  assert.ok(toolNames.includes("docs.get_pack"));
  assert.ok(toolNames.includes("docs.error_help"));

  for (const t of tools) {
    assert.equal(t.category, "read");
    assert.equal(t.risk_level, "safe");
    assert.ok(t.description.length > 5);
  }
});

test("DocsToolProvider executes docs.search, docs.read, docs.list_recipes, docs.get_pack, docs.error_help", async () => {
  const provider = new DocsToolProvider(docsRoot);

  // 1. Search tool
  const searchRes = await provider.executeTool("docs.search", { query: "coyote time", limit: 3 });
  assert.equal(searchRes.is_error, false);
  const searchData = searchRes.content as { results: Array<{ id: string; score: number }> };
  assert.ok(searchData.results.length > 0);
  assert.ok(searchData.results.some((r) => r.id.includes("player-movement-platformer") || r.id.includes("game-feel")));

  // 2. Read tool
  const readRes = await provider.executeTool("docs.read", { id: "agents.mental-model" });
  assert.equal(readRes.is_error, false);
  const readData = readRes.content as { id: string; content: string; token_count: number };
  assert.equal(readData.id, "agents.mental-model");
  assert.ok(readData.content.includes("bevy_ecs"));
  assert.ok(readData.token_count > 0);

  // 3. List recipes tool
  const recipesRes = await provider.executeTool("docs.list_recipes", {});
  assert.equal(recipesRes.is_error, false);
  const recipesData = recipesRes.content as { recipes: Array<{ id: string }>; total: number };
  assert.ok(recipesData.total >= 30);

  // 4. Get pack tool
  const packRes = await provider.executeTool("docs.get_pack", { topic: "platformer-mechanics" });
  assert.equal(packRes.is_error, false);
  const packData = packRes.content as { topic: string; content: string };
  assert.equal(packData.topic, "platformer-mechanics");
  assert.ok(packData.content.length > 100);

  // 5. Error help tool
  const errorRes = await provider.executeTool("docs.error_help", { code: "-32001" });
  assert.equal(errorRes.is_error, false);
  const errorData = errorRes.content as { code: number; meaning: string; fix: string };
  assert.equal(errorData.meaning, "Unauthorized");
});

test("Evaluation set: docs.search returns the expected page in top 3 for >= 90% of 40 agent questions", async () => {
  const provider = new DocsToolProvider(docsRoot);
  assert.equal(EVALUATION_QUESTIONS.length, 40, "Evaluation questions set must contain exactly 40 questions");

  let top1Hits = 0;
  let top3Hits = 0;
  const failures: Array<{ question: string; expected: string; got: string[] }> = [];

  for (const item of EVALUATION_QUESTIONS) {
    const results = provider.search(item.question, 3);
    const topIds = results.map((r) => r.id);

    if (topIds.length > 0 && topIds[0] === item.expectedDocId) {
      top1Hits++;
    }

    if (topIds.includes(item.expectedDocId)) {
      top3Hits++;
    } else {
      failures.push({
        question: item.question,
        expected: item.expectedDocId,
        got: topIds,
      });
    }
  }

  const top3Accuracy = (top3Hits / EVALUATION_QUESTIONS.length) * 100;
  console.log(`Docs Search Evaluation: Top-1 = ${top1Hits}/40, Top-3 = ${top3Hits}/40 (${top3Accuracy.toFixed(1)}%)`);

  if (failures.length > 0) {
    console.log("Evaluation misses in top 3:", JSON.stringify(failures, null, 2));
  }

  assert.ok(
    top3Accuracy >= 90.0,
    `Search quality must achieve >= 90% in top 3 results, got ${top3Accuracy.toFixed(1)}%`
  );
});
