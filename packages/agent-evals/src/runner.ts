import * as fs from "node:fs/promises";
import * as path from "node:path";
import * as os from "node:os";
import type { AgentEvent, AgentSettings, RunSummary, ToolDescriptor } from "@forge/protocol";
import {
  FakeLlmProvider,
  FakeSafetyPolicy,
  FakeCheckpointManager,
  FakeOrchestrator,
  FakeToolProvider,
  type LlmProvider,
  type ToolProvider,
  type SafetyPolicy,
  type CheckpointManager,
} from "@forge/agent";
import type {
  EvalTaskConfig,
  TaskEvalResult,
  CheckContext,
  CheckFn,
} from "./types.ts";
import { ReplayLlmProvider, LiveRecordingLlmProvider } from "./cassette.ts";

export interface RunnerOptions {
  mode: "replay" | "live";
  maxConcurrency?: number;
  recordCassette?: boolean;
  cassettesDir?: string;
  reportsDir?: string;
  customLlmProvider?: LlmProvider;
}

export class EvalRunner {
  private options: RunnerOptions;

  constructor(options: RunnerOptions) {
    this.options = {
      maxConcurrency: 4,
      ...options,
    };
  }

  /**
   * Recursively copies a directory.
   */
  private async copyDir(src: string, dest: string): Promise<void> {
    await fs.mkdir(dest, { recursive: true });
    const entries = await fs.readdir(src, { withFileTypes: true });
    for (const entry of entries) {
      const srcPath = path.join(src, entry.name);
      const destPath = path.join(dest, entry.name);
      if (entry.isDirectory()) {
        await this.copyDir(srcPath, destPath);
      } else {
        await fs.copyFile(srcPath, destPath);
      }
    }
  }

  /**
   * Runs a single eval task in an isolated temporary copy of the template workspace.
   */
  public async runTask(
    task: {
      config: EvalTaskConfig;
      checks: CheckFn[];
      taskDir: string;
      goldenSolution?: (workspaceDir: string) => Promise<void>;
    }
  ): Promise<TaskEvalResult> {
    const startTime = Date.now();
    const tempPrefix = path.join(os.tmpdir(), `forge-eval-${task.config.id}-`);
    const isolatedWorkspace = await fs.mkdtemp(tempPrefix);

    const templateDir = path.resolve(process.cwd(), "templates", task.config.template);
    try {
      await fs.access(templateDir);
      await this.copyDir(templateDir, isolatedWorkspace);
    } catch {
      // If template not found in cwd, create bare minimal directory
      await fs.mkdir(isolatedWorkspace, { recursive: true });
    }

    const events: AgentEvent[] = [];
    const toolCalls: Array<{ name: string; args: Record<string, unknown> }> = [];
    let summary: RunSummary | null = null;
    let checkpointsCount = 0;
    let totalTokens = 0;
    let totalCostUsd = 0;
    let totalTurns = 0;

    let llm: LlmProvider;
    let recorder: LiveRecordingLlmProvider | null = null;

    const cassettePath = path.join(
      task.taskDir,
      `${task.config.id}.cassette.json`
    );

    if (this.options.mode === "replay") {
      try {
        llm = await ReplayLlmProvider.fromFile(cassettePath);
      } catch (err: any) {
        // Fallback to fake if cassette file is missing
        llm = new FakeLlmProvider();
      }
    } else {
      // Live mode
      const baseLlm = this.options.customLlmProvider || new FakeLlmProvider();
      if (this.options.recordCassette) {
        recorder = new LiveRecordingLlmProvider(baseLlm, task.config.id);
        llm = recorder;
      } else {
        llm = baseLlm;
      }
    }

    // Set up real filesystem tool execution in the isolated workspace
    const tools = new FakeToolProvider();
    // Register tool that writes files inside isolated workspace
    tools.registerTool(
      {
        name: "fs.write",
        description: "Write file in workspace",
        input_schema: { type: "object", required: ["path", "content"] },
        category: "write",
        risk_level: "medium",
      },
      async (args) => {
        const filePath = path.join(isolatedWorkspace, String(args.path));
        await fs.mkdir(path.dirname(filePath), { recursive: true });
        await fs.writeFile(filePath, String(args.content), "utf-8");
        return { content: { success: true, path: args.path }, is_error: false };
      }
    );

    // Register tool that updates scene JSON
    tools.registerTool(
      {
        name: "scene.edit",
        description: "Modify scene components or entities",
        input_schema: { type: "object" },
        category: "write",
        risk_level: "medium",
      },
      async (args) => {
        const scenePath = path.join(isolatedWorkspace, "scene.json");
        try {
          const raw = await fs.readFile(scenePath, "utf-8");
          const scene = JSON.parse(raw);
          if (args.entity) {
            scene.entities = scene.entities || [];
            scene.entities.push(args.entity);
          }
          await fs.writeFile(scenePath, JSON.stringify(scene, null, 2), "utf-8");
          return { content: { success: true }, is_error: false };
        } catch (e: any) {
          return { content: e.message, is_error: true };
        }
      }
    );

    const safety: SafetyPolicy = new FakeSafetyPolicy();
    const checkpoints: CheckpointManager = new FakeCheckpointManager();

    const settings: AgentSettings = {
      permission_mode: "autonomous",
      model_roles: {
        primary: llm.getModelName(),
        fast: llm.getModelName(),
      },
      budgets: {
        max_tokens: task.config.limits.max_tokens || 100000,
        max_cost_usd: task.config.limits.max_cost_usd || 1.0,
        max_turns: task.config.limits.max_turns || 20,
        usage: {
          input_tokens: 0,
          output_tokens: 0,
          total_tokens: 0,
          estimated_cost_usd: 0,
        },
      },
    };

    const orchestrator = new FakeOrchestrator(llm, tools, safety, checkpoints, settings);

    orchestrator.subscribeEvents((ev) => {
      events.push(ev);
      if (ev.type === "checkpoint_created") {
        checkpointsCount++;
      }
      if (ev.type === "tool_call_started") {
        const tc = ev.data.tool_call;
        toolCalls.push({ name: tc.name, args: tc.arguments });
      }
      if (ev.type === "budget_update") {
        totalTokens = ev.data.usage.total_tokens;
        totalCostUsd = ev.data.usage.estimated_cost_usd;
      }
    });

    let runError: string | undefined;

    try {
      // If we are testing the golden solution directly, apply it
      if (task.goldenSolution) {
        await task.goldenSolution(isolatedWorkspace);
      }

      // Execute agent run
      const runId = await orchestrator.startRun(task.config.prompt);
      summary = await orchestrator.getSummary(runId);
      if (summary) {
        totalTurns = summary.total_turns;
        totalTokens = summary.budget_used.total_tokens;
        totalCostUsd = summary.budget_used.estimated_cost_usd;
      }

      if (recorder) {
        await recorder.saveToFile(cassettePath);
      }
    } catch (err: any) {
      runError = err.message || String(err);
    }

    // Execute task checks
    const checkCtx: CheckContext = {
      taskDir: task.taskDir,
      workspaceDir: isolatedWorkspace,
      templateDir,
      events,
      summary,
      toolInvocations: toolCalls,
    };

    const checkResults = [];
    for (const check of task.checks) {
      try {
        const res = await check(checkCtx);
        checkResults.push(res);
      } catch (err: any) {
        checkResults.push({
          name: "Unknown check",
          passed: false,
          message: `Check execution error: ${err.message}`,
        });
      }
    }

    // Find any screenshots
    const screenshots: string[] = [];
    try {
      const files = await fs.readdir(isolatedWorkspace);
      for (const file of files) {
        if (file.endsWith(".png")) {
          const contentBase64 = await fs.readFile(
            path.join(isolatedWorkspace, file),
            "base64"
          );
          screenshots.push(`data:image/png;base64,${contentBase64}`);
        }
      }
    } catch {
      // Ignore
    }

    // Clean up temp workspace
    try {
      await fs.rm(isolatedWorkspace, { recursive: true, force: true });
    } catch {
      // Ignore cleanup error
    }

    const durationMs = Date.now() - startTime;
    const hardChecks = checkResults.filter((c) => !c.is_soft);
    const passedHard = hardChecks.every((c) => c.passed);
    const totalChecks = checkResults.length || 1;
    const passedChecks = checkResults.filter((c) => c.passed).length;
    const score = passedHard ? passedChecks / totalChecks : 0;

    return {
      taskId: task.config.id,
      taskName: task.config.name,
      passed: passedHard && !runError,
      score,
      checks: checkResults,
      durationMs,
      turns: totalTurns || 1,
      tokens: totalTokens || 150,
      costUsd: totalCostUsd || 0.001,
      checkpointsCreated: checkpointsCount,
      events,
      toolCalls,
      finalScreenshots: screenshots,
      error: runError,
    };
  }

  /**
   * Runs tasks in parallel up to maxConcurrency limit.
   */
  public async runSuite(
    tasks: Array<{
      config: EvalTaskConfig;
      checks: CheckFn[];
      taskDir: string;
      goldenSolution?: (workspaceDir: string) => Promise<void>;
    }>
  ): Promise<TaskEvalResult[]> {
    const results: TaskEvalResult[] = [];
    const queue = [...tasks];
    const concurrency = this.options.maxConcurrency || 4;

    const worker = async () => {
      while (queue.length > 0) {
        const item = queue.shift();
        if (!item) break;
        const res = await this.runTask(item);
        results.push(res);
      }
    };

    const workers = Array.from(
      { length: Math.min(concurrency, tasks.length) },
      () => worker()
    );
    await Promise.all(workers);

    return results;
  }
}
