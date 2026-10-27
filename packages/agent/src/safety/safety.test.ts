import test from "node:test";
import assert from "node:assert/strict";
import * as fs from "node:fs/promises";
import * as path from "node:path";
import * as os from "node:os";
import type { ToolDescriptor, AgentSettings } from "@forge/protocol";
import {
  SecretFilter,
  BudgetTracker,
  PermissionsManager,
  StandardSafetyPolicy,
  GitCheckpointManager,
} from "./index.ts";

async function makeTempDir(prefix: string): Promise<string> {
  return await fs.mkdtemp(path.join(os.tmpdir(), prefix));
}

async function removeTempDir(dir: string): Promise<void> {
  try {
    await fs.rm(dir, { recursive: true, force: true });
  } catch {
    // ignore
  }
}

// -----------------------------------------------------------------------------
// 1. SecretFilter Tests
// -----------------------------------------------------------------------------
test("SecretFilter redacts API keys, tokens, and private keys from strings and objects", () => {
  const filter = new SecretFilter();

  const openAiKey = "sk-1234567890abcdef1234567890abcdef";
  const geminiKey = "AIzaSyD-1234567890abcdef1234567890abc";
  const githubToken = "ghp_1234567890abcdef1234567890abcdef12";
  const sampleLog = `Starting provider with key: ${openAiKey} and backup: ${geminiKey} and token: ${githubToken}`;

  const redacted = filter.redact(sampleLog);
  assert.ok(!redacted.includes(openAiKey));
  assert.ok(!redacted.includes(geminiKey));
  assert.ok(!redacted.includes(githubToken));
  assert.ok(redacted.includes("[REDACTED_OPENAI_API_KEY]"));
  assert.ok(redacted.includes("[REDACTED_GOOGLE_API_KEY]"));
  assert.ok(redacted.includes("[REDACTED_GITHUB_TOKEN]"));

  // Redact object
  const complexObj = {
    auth: {
      token: "Bearer ya29.1234567890abcdef1234567890",
      key: openAiKey,
    },
    count: 42,
  };
  const redactedObj = filter.redactObject(complexObj);
  assert.equal(redactedObj.count, 42);
  assert.equal(redactedObj.auth.token, "Bearer [REDACTED]");
  assert.equal(redactedObj.auth.key, "[REDACTED_OPENAI_API_KEY]");
});

// -----------------------------------------------------------------------------
// 2. BudgetTracker Tests
// -----------------------------------------------------------------------------
test("BudgetTracker warns near limit, pauses/exceeds when limit reached, and resumes after extension", () => {
  const tracker = new BudgetTracker({
    max_tokens: 1000,
    max_cost_usd: 1.0,
    max_steps: 10,
    max_tool_calls: 5,
    max_consecutive_errors: 3,
    warning_threshold: 0.8,
  });

  // Safe initially
  assert.equal(tracker.check().exceeded, false);
  assert.equal(tracker.check().near_limit, false);

  // Near limit warning at 80% (800 tokens)
  tracker.addUsage({ tokens: 850 });
  const warnCheck = tracker.check();
  assert.equal(warnCheck.exceeded, false);
  assert.equal(warnCheck.near_limit, true);
  assert.ok(warnCheck.reason?.includes("warning"));

  const warnEvent = tracker.createBudgetUpdateEvent("run_1", warnCheck.reason);
  assert.equal(warnEvent.type, "budget_update");

  // Exceed limit (over 1000)
  tracker.addUsage({ tokens: 200 });
  const exceedCheck = tracker.check();
  assert.equal(exceedCheck.exceeded, true);
  assert.ok(exceedCheck.reason?.includes("exceeded"));

  // Resume run by extending budget
  tracker.extendBudget({ max_tokens: 1000 });
  const resumedCheck = tracker.check();
  assert.equal(resumedCheck.exceeded, false);

  // Consecutive error exhaustion
  tracker.addUsage({ is_error: true });
  tracker.addUsage({ is_error: true });
  assert.equal(tracker.check().exceeded, false);
  tracker.addUsage({ is_error: true });
  const errorCheck = tracker.check();
  assert.equal(errorCheck.exceeded, true);
  assert.equal(errorCheck.exceeded_dimension, "consecutive_errors");

  // Reset consecutive errors after a successful call
  tracker.addUsage({ is_error: false });
  assert.equal(tracker.check().exceeded, false);
});

// -----------------------------------------------------------------------------
// 3. SafetyPolicy & Hard Deny List Tests
// -----------------------------------------------------------------------------
test("StandardSafetyPolicy strictly enforces hard deny list regardless of settings or mode", async () => {
  const tempDir = await makeTempDir("forge-safety-deny-");
  try {
    const policy = new StandardSafetyPolicy({ projectRoot: tempDir });

    const fullAutoSettings: AgentSettings = {
      permission_mode: "autonomous",
      model_roles: {},
      budgets: {
        max_tokens: 100000,
        max_cost_usd: 10,
        max_turns: 50,
        usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0, estimated_cost_usd: 0 },
      },
    };

    const toolWrite: ToolDescriptor = {
      name: "fs.write",
      description: "Write file",
      input_schema: {},
      category: "write",
      risk_level: "low",
    };

    // 1. Outside project path
    const outsideRes = await policy.evaluate(
      toolWrite,
      { path: path.join(tempDir, "..", "secret.txt") },
      fullAutoSettings
    );
    assert.equal(outsideRes.allowed, false);
    assert.equal(outsideRes.risk_level, "critical");

    // 2. .git internals
    const gitRes = await policy.evaluate(
      toolWrite,
      { path: path.join(tempDir, ".git", "config") },
      fullAutoSettings
    );
    assert.equal(gitRes.allowed, false);
    assert.equal(gitRes.risk_level, "critical");

    // 3. .forge/secrets
    const secretsRes = await policy.evaluate(
      toolWrite,
      { path: path.join(tempDir, ".forge", "secrets", "api_key.txt") },
      fullAutoSettings
    );
    assert.equal(secretsRes.allowed, false);
    assert.equal(secretsRes.risk_level, "critical");

    // 4. Deleting project root
    const rootDeleteRes = await policy.evaluate(
      { ...toolWrite, name: "fs.delete" },
      { path: tempDir },
      fullAutoSettings
    );
    assert.equal(rootDeleteRes.allowed, false);
    assert.equal(rootDeleteRes.risk_level, "critical");

    // Even if an always-allow rule exists, hard deny MUST take precedence
    await policy.getPermissionsManager().addRule({ category: "write", path_glob: "**/*" });
    const blockedEvenWithRule = await policy.evaluate(
      toolWrite,
      { path: path.join(tempDir, ".git", "HEAD") },
      fullAutoSettings
    );
    assert.equal(blockedEvenWithRule.allowed, false);
  } finally {
    await removeTempDir(tempDir);
  }
});

test("StandardSafetyPolicy permission modes: Ask, Auto-edit, Full auto", async () => {
  const tempDir = await makeTempDir("forge-safety-modes-");
  try {
    const policy = new StandardSafetyPolicy({ projectRoot: tempDir });

    const toolRead: ToolDescriptor = {
      name: "fs.read",
      description: "Read file",
      input_schema: {},
      category: "read",
      risk_level: "safe",
    };

    const toolWrite: ToolDescriptor = {
      name: "fs.write",
      description: "Write file",
      input_schema: {},
      category: "write",
      risk_level: "low",
    };

    const toolExec: ToolDescriptor = {
      name: "shell.exec",
      description: "Run shell command",
      input_schema: {},
      category: "exec",
      risk_level: "medium",
    };

    const baseBudgets = {
      max_tokens: 100000,
      max_cost_usd: 10,
      max_turns: 50,
      usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0, estimated_cost_usd: 0 },
    };

    // Ask mode (ask_all): read is allowed; write and exec ask
    const askSettings: AgentSettings = { permission_mode: "ask_all", model_roles: {}, budgets: baseBudgets };
    const askRead = await policy.evaluate(toolRead, { path: "hello.txt" }, askSettings);
    assert.equal(askRead.requires_permission, false);

    const askWrite = await policy.evaluate(toolWrite, { path: "hello.txt" }, askSettings);
    assert.equal(askWrite.requires_permission, true);

    const askExec = await policy.evaluate(toolExec, { command: "npm test" }, askSettings);
    assert.equal(askExec.requires_permission, true);

    // Auto-edit mode (ask_risky): write is automatic, exec asks
    const autoEditSettings: AgentSettings = { permission_mode: "ask_risky", model_roles: {}, budgets: baseBudgets };
    const autoEditWrite = await policy.evaluate(toolWrite, { path: "hello.txt" }, autoEditSettings);
    assert.equal(autoEditWrite.requires_permission, false);

    const autoEditExec = await policy.evaluate(toolExec, { command: "npm test" }, autoEditSettings);
    assert.equal(autoEditExec.requires_permission, true);

    // Full auto (autonomous): everything allowed automatically
    const fullAutoSettings: AgentSettings = { permission_mode: "autonomous", model_roles: {}, budgets: baseBudgets };
    const fullAutoExec = await policy.evaluate(toolExec, { command: "npm test" }, fullAutoSettings);
    assert.equal(fullAutoExec.requires_permission, false);
  } finally {
    await removeTempDir(tempDir);
  }
});

test("StandardSafetyPolicy persistent permissions and permission flow with timeout, pause, resume, stop", async () => {
  const tempDir = await makeTempDir("forge-safety-flow-");
  try {
    const policy = new StandardSafetyPolicy({ projectRoot: tempDir, defaultTimeoutMs: 100 });

    // 1. Timeout denies
    const timeoutDecision = await policy.requestPermission({
      id: "req_timeout",
      tool_name: "shell.exec",
      risk_level: "medium",
      description: "execute command",
      arguments: { command: "cargo build" },
    }, 50);
    assert.equal(timeoutDecision, "deny");

    // 2. Resolve approve_once
    const promiseApprove = policy.requestPermission({
      id: "req_approve",
      tool_name: "shell.exec",
      risk_level: "medium",
      description: "execute command",
      arguments: { command: "cargo test" },
    }, 2000);

    const resolved = policy.resolvePermission("req_approve", "approve_once");
    assert.equal(resolved, true);
    assert.equal(await promiseApprove, "approve_once");

    // 3. Stop denies pending requests
    const promiseStop = policy.requestPermission({
      id: "req_stop",
      tool_name: "shell.exec",
      risk_level: "medium",
      description: "execute command",
      arguments: {},
    }, 5000);
    policy.stop();
    assert.equal(await promiseStop, "deny");

    // 4. Persistence of always_allow
    const pm = policy.getPermissionsManager();
    await pm.addRule({
      tool_name: "fs.write",
      path_glob: "scripts/*.ts",
    });

    const persistedRules = await pm.getRules();
    assert.equal(persistedRules.length, 1);
    assert.equal(persistedRules[0].tool_name, "fs.write");

    // Verify .forge/permissions.json on disk
    const storedOnDisk = await fs.readFile(path.join(tempDir, ".forge", "permissions.json"), "utf8");
    assert.ok(storedOnDisk.includes("scripts/*.ts"));
  } finally {
    await removeTempDir(tempDir);
  }
});

// -----------------------------------------------------------------------------
// 4. GitCheckpointManager Tests
// -----------------------------------------------------------------------------
test("GitCheckpointManager handles not-a-repo case and dirty working tree", async () => {
  const tempDir = await makeTempDir("forge-git-init-");
  try {
    // Write an uncommitted user file before init
    await fs.writeFile(path.join(tempDir, "user_draft.txt"), "Initial user thoughts");

    const manager = new GitCheckpointManager({ projectRoot: tempDir, uncommittedStrategy: "snapshot" });
    const cp = await manager.createCheckpoint("run_101", "Completed step 1: create player", ["player.ts"]);

    assert.ok(cp.id.startsWith("chk_"));
    assert.equal(cp.run_id, "run_101");

    // Verify user draft was preserved in git repo
    const userFile = await fs.readFile(path.join(tempDir, "user_draft.txt"), "utf8");
    assert.equal(userFile, "Initial user thoughts");

    // List checkpoints
    const list = await manager.listCheckpoints("run_101");
    assert.equal(list.length, 1);
    assert.equal(list[0].id, cp.id);
  } finally {
    await removeTempDir(tempDir);
  }
});

test("GitCheckpointManager checkpoint, revert round trip without losing later user edits, and squash-to-main", async () => {
  const tempDir = await makeTempDir("forge-git-roundtrip-");
  try {
    const manager = new GitCheckpointManager({ projectRoot: tempDir });

    // Step 1: Create file A and checkpoint 1
    await fs.writeFile(path.join(tempDir, "featureA.ts"), "export const A = 1;\n");
    const cp1 = await manager.createCheckpoint("run_202", "feat: implement feature A");

    // Step 2: Create file B and checkpoint 2
    await fs.writeFile(path.join(tempDir, "featureB.ts"), "export const B = 2;\n");
    const cp2 = await manager.createCheckpoint("run_202", "feat: implement feature B");

    // Step 3: User makes edits after checkpoint 2
    await fs.writeFile(path.join(tempDir, "user_notes.txt"), "User notes that must not be deleted");

    // Check diff summary of cp2
    const diff = await manager.getCheckpointDiff(cp2.git_ref);
    assert.ok(diff.files_changed >= 1);
    assert.ok(diff.insertions >= 1);

    // Revert to checkpoint 1
    const reverted = await manager.revertToCheckpoint(cp1.id);
    assert.equal(reverted, true);

    // Verify featureA is still there, featureB is restored to cp1 state (does not exist in cp1)
    const existsA = await fs.access(path.join(tempDir, "featureA.ts")).then(() => true).catch(() => false);
    const existsB = await fs.access(path.join(tempDir, "featureB.ts")).then(() => true).catch(() => false);
    const existsNotes = await fs.access(path.join(tempDir, "user_notes.txt")).then(() => true).catch(() => false);

    assert.equal(existsA, true);
    assert.equal(existsB, false);
    // User notes should NOT be lost
    assert.equal(existsNotes, true);

    // Squash to main
    const squashHash = await manager.squashToMain();
    assert.ok(squashHash);
  } finally {
    await removeTempDir(tempDir);
  }
});
