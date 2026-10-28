import { test } from "node:test";
import assert from "node:assert";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { buildDocumentation } from "../build.ts";
import { validateAllQualityGates } from "../validate.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, "../../..");
const DOCS_DIST = path.join(REPO_ROOT, "docs/dist");

test("docgen builds documentation outputs and validates quality gates", async () => {
  // Execute full docgen build
  await buildDocumentation();

  // Validate quality gates
  const gates = validateAllQualityGates();
  assert.strictEqual(gates.success, true, "All quality gates should pass cleanly");

  // Check outputs in docs/dist
  assert.ok(fs.existsSync(path.join(DOCS_DIST, "llms.txt")), "docs/dist/llms.txt must exist");
  assert.ok(fs.existsSync(path.join(DOCS_DIST, "llms-full.txt")), "docs/dist/llms-full.txt must exist");
  assert.ok(fs.existsSync(path.join(DOCS_DIST, "index.json")), "docs/dist/index.json must exist");
  assert.ok(fs.existsSync(path.join(DOCS_DIST, "search-index.json")), "docs/dist/search-index.json must exist");
  assert.ok(fs.existsSync(path.join(DOCS_DIST, "docs-bundle.zip")), "docs/dist/docs-bundle.zip must exist");
  assert.ok(fs.existsSync(path.join(DOCS_DIST, "human-bundle")), "docs/dist/human-bundle must exist");

  // Validate index.json structure
  const indexContent = JSON.parse(fs.readFileSync(path.join(DOCS_DIST, "index.json"), "utf-8"));
  assert.ok(indexContent.pages.length > 20, "Index must contain all generated and hand-authored pages");
  assert.ok(indexContent.tools.length > 0, "Index must contain bridge tools");
  assert.ok(indexContent.components.length > 0, "Index must contain components");
  assert.ok(indexContent.agent_packs.length > 0, "Index must contain agent packs");

  // Validate agent packs size limit
  const agentPacksDir = path.join(DOCS_DIST, "agent-packs");
  const packs = fs.readdirSync(agentPacksDir).filter(f => f.endsWith(".md"));
  assert.ok(packs.length >= 9, "Must generate at least 9 agent packs");

  for (const pack of packs) {
    const content = fs.readFileSync(path.join(agentPacksDir, pack), "utf-8");
    const tokenEst = Math.ceil(content.length / 4);
    assert.ok(tokenEst <= 3000, `Agent pack ${pack} must be under 3000 tokens (got ${tokenEst})`);
  }
});
