import { describe, it, expect, beforeEach } from "vitest";
import {
  useAgentStore,
  selectAgentStatus,
  selectAgentMessages,
  selectCurrentPlan,
  selectToolCalls,
  selectPendingPermissions,
  selectCheckpoints,
  selectBudgetUsage,
} from "../state/agent";
import fs from "node:fs";
import path from "node:path";

const FIXTURES_DIR = path.resolve(__dirname, "../../fixtures/agent");

function readFixture(name: string): string {
  return fs.readFileSync(path.join(FIXTURES_DIR, name), "utf-8");
}

describe("Agent Store with Fixtures", () => {
  beforeEach(() => {
    // Reset store state
    useAgentStore.setState({
      runId: null,
      status: "stopped",
      crashedReason: null,
      messages: [],
      currentPlan: null,
      toolCalls: {},
      pendingPermissions: [],
      checkpoints: [],
      budgetUsage: null,
      settings: null,
      client: null,
      isFixtureMode: true,
    });
  });

  it("replays short_answer fixture and accumulates streamed message chunks", async () => {
    await useAgentStore.getState().init({ fixtureMode: true });
    const fixtureContent = readFixture("short_answer.jsonl");

    await useAgentStore.getState().replayFixture(fixtureContent, 10);

    const messages = selectAgentMessages(useAgentStore.getState());
    expect(messages.length).toBe(2);
    expect(messages[0].role).toBe("user");
    expect(messages[0].content).toBe("What is Forge?");
    expect(messages[1].role).toBe("agent");
    expect(messages[1].content).toBe(
      "Forge is an AI-native game engine and Windows Studio IDE."
    );
  });

  it("replays tool_calls_and_permission fixture: verifies plan, tools, checkpoints, and permissions", async () => {
    await useAgentStore.getState().init({ fixtureMode: true });
    const fixtureContent = readFixture("tool_calls_and_permission.jsonl");

    await useAgentStore.getState().replayFixture(fixtureContent, 10);

    const state = useAgentStore.getState();
    const plan = selectCurrentPlan(state);
    expect(plan).not.toBeNull();
    expect(plan?.id).toBe("plan_01");
    expect(plan?.steps.length).toBe(2);
    expect(plan?.steps[0].status).toBe("completed");
    expect(plan?.steps[1].status).toBe("completed");

    const tools = selectToolCalls(state);
    expect(tools.length).toBe(2);
    expect(tools.find((t) => t.name === "script.read")?.status).toBe("completed");
    expect(tools.find((t) => t.name === "script.write")?.status).toBe("completed");

    const checkpoints = selectCheckpoints(state);
    expect(checkpoints.length).toBe(1);
    expect(checkpoints[0].id).toBe("chk_01");
    expect(checkpoints[0].affected_files).toContain("scripts/player.ts");

    const budget = selectBudgetUsage(state);
    expect(budget).not.toBeNull();
    expect(budget?.total_tokens).toBe(205);
    expect(budget?.estimated_cost_usd).toBe(0.0008);

    // Permission was requested then resolved, so pendingPermissions should be empty
    const permissions = selectPendingPermissions(state);
    expect(permissions.length).toBe(0);
  });

  it("replays failed_run fixture and records error message", async () => {
    await useAgentStore.getState().init({ fixtureMode: true });
    const fixtureContent = readFixture("failed_run.jsonl");

    await useAgentStore.getState().replayFixture(fixtureContent, 10);

    const messages = selectAgentMessages(useAgentStore.getState());
    expect(messages.length).toBe(2);
    expect(messages[0].role).toBe("user");
    expect(messages[1].role).toBe("system");
    expect(messages[1].content).toContain(
      "Execution policy violation: unauthorized tool invocation prohibited"
    );
  });

  it("transitions to clear Crashed state and reason when sidecar crashes", async () => {
    await useAgentStore.getState().init({ fixtureMode: true });
    expect(selectAgentStatus(useAgentStore.getState())).toBe("running");

    useAgentStore
      .getState()
      .setFixtureCrashed("Process terminated unexpectedly with exit code 3221225477");

    const status = selectAgentStatus(useAgentStore.getState());
    expect(status).toBe("crashed");
    expect(useAgentStore.getState().crashedReason).toBe(
      "Process terminated unexpectedly with exit code 3221225477"
    );
  });

  it("handles permission request resolution", async () => {
    await useAgentStore.getState().init({ fixtureMode: true });
    const store = useAgentStore.getState();

    store.applyEvent({
      type: "permission_requested",
      run_id: "run_perm_test",
      timestamp: 12345,
      data: {
        request: {
          id: "req_test_1",
          tool_name: "script.write",
          risk_level: "medium",
          description: "Modify script",
          arguments: { path: "script.ts" },
          suggested_diff: null,
        },
      },
    });

    expect(selectPendingPermissions(useAgentStore.getState()).length).toBe(1);

    await useAgentStore.getState().resolvePermission("req_test_1", true);
    expect(selectPendingPermissions(useAgentStore.getState()).length).toBe(0);
  });
});
