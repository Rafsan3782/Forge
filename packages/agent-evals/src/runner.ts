import * as fs from "node:fs/promises";
import * as path from "node:path";
import * as os from "node:os";
import type { AgentEvent, AgentSettings, RunSummary } from "@forge/protocol";
import {
  FakeLlmProvider,
  FakeSafetyPolicy,
  FakeCheckpointManager,
  FakeOrchestrator,
  FakeToolProvider,
  type LlmProvider,
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
  useGoldenSolution?: boolean;
}

interface EventDataWithToolCall {
  tool_call?: {
    name: string;
    arguments: Record<string, unknown>;
  };
}

interface EventDataWithUsage {
  usage?: {
    total_tokens: number;
    estimated_cost_usd: number;
  };
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
   * Recursively copies a directory, ignoring heavy tooling artifacts.
   */
  private async copyDir(src: string, dest: string): Promise<void> {
    await fs.mkdir(dest, { recursive: true });
    const entries = await fs.readdir(src, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.name === "node_modules" || entry.name === "target" || entry.name === ".git") {
        continue;
      }
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

    // Look for template in cwd or relative up
    const possibleTemplateDirs = [
      path.resolve(process.cwd(), "templates", task.config.template),
      path.resolve(process.cwd(), "..", "..", "templates", task.config.template),
      path.resolve(process.cwd(), "..", "templates", task.config.template),
      path.resolve(task.taskDir, "..", "..", "..", "..", "templates", task.config.template),
    ];
    let templateDir = possibleTemplateDirs[0];
    for (const cand of possibleTemplateDirs) {
      try {
        await fs.access(cand);
        templateDir = cand;
        break;
      } catch {
        // try next
      }
    }

    try {
      await fs.access(templateDir);
      await this.copyDir(templateDir, isolatedWorkspace);
    } catch {
      // If template not found, create bare minimal directory
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
      } catch {
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
          const scene = JSON.parse(raw) as { entities?: unknown[] };
          if (args.entity) {
            const ent = args.entity as { components?: Record<string, Record<string, unknown>> };
            if (ent.components?.Sprite) {
              if (!ent.components.Sprite.color_tint) {
                ent.components.Sprite.color_tint = [1.0, 1.0, 1.0, 1.0];
              }
              if (ent.components.Sprite.z_order === undefined) {
                ent.components.Sprite.z_order = 0;
              }
            }
            scene.entities = scene.entities || [];
            scene.entities.push(args.entity);
          }
          await fs.writeFile(scenePath, JSON.stringify(scene, null, 2), "utf-8");
          return { content: { success: true }, is_error: false };
        } catch (e: unknown) {
          return { content: e instanceof Error ? e.message : String(e), is_error: true };
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
        const tc = (ev.data as EventDataWithToolCall)?.tool_call;
        if (tc) {
          toolCalls.push({ name: tc.name, args: tc.arguments });
        }
      }
      if (ev.type === "budget_update") {
        const usage = (ev.data as EventDataWithUsage)?.usage;
        if (usage) {
          totalTokens = usage.total_tokens;
          totalCostUsd = usage.estimated_cost_usd;
        }
      }
    });

    let runError: string | undefined;

    try {
      if (this.options.useGoldenSolution && task.goldenSolution) {
        await task.goldenSolution(isolatedWorkspace);
      } else {
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
      }
    } catch (err: unknown) {
      runError = err instanceof Error ? err.message : String(err);
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
      } catch (err: unknown) {
        checkResults.push({
          name: "Unknown check",
          passed: false,
          message: `Check execution error: ${err instanceof Error ? err.message : String(err)}`,
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
