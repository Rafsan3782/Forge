import type {
  Plan,
  PlanStep,
  ToolDescriptor,
  PermissionRequest,
  Checkpoint,
  Budget,
  BudgetUsage,
  AgentSettings,
  AgentEvent,
  AgentCommand,
  RunSummary,
} from "@forge/protocol";

import type {
  LlmProvider,
  LlmMessage,
  LlmTurnChunk,
  LlmTurnOptions,
  ToolProvider,
  ToolResult,
  SafetyPolicy,
  SafetyCheckResult,
  CheckpointManager,
  Orchestrator,
  SkillProvider,
  SkillDefinition,
  ProjectBrain,
  DocsProvider,
} from "./interfaces.ts";

// -----------------------------------------------------------------------------
// In-Memory Fake: LlmProvider
// -----------------------------------------------------------------------------

export interface ScriptedTurn {
  chunks: LlmTurnChunk[];
}

export class FakeLlmProvider implements LlmProvider {
  private cannedTurns: ScriptedTurn[] = [];
  private callCount = 0;
  private recordedCalls: Array<{ messages: LlmMessage[]; options?: LlmTurnOptions }> = [];

  constructor(turns?: ScriptedTurn[]) {
    if (turns) {
      this.cannedTurns = [...turns];
    }
  }

  public getModelName(): string {
    return "fake-gemini-2.5";
  }

  public enqueueTurn(chunks: LlmTurnChunk[]): void {
    this.cannedTurns.push({ chunks });
  }

  public getRecordedCalls() {
    return this.recordedCalls;
  }

  public async *streamTurn(
    messages: LlmMessage[],
    options?: LlmTurnOptions
  ): AsyncIterable<LlmTurnChunk> {
    this.recordedCalls.push({ messages, options });
    const turnIndex = this.callCount++;

    if (turnIndex < this.cannedTurns.length) {
      const turn = this.cannedTurns[turnIndex];
      for (const chunk of turn.chunks) {
        yield chunk;
      }
    } else {
      // Default deterministic response
      yield {
        thought: "Processing user request deterministically.",
        delta: "I understand your request.",
        finish_reason: "stop",
        usage: {
          input_tokens: 10,
          output_tokens: 5,
          total_tokens: 15,
          estimated_cost_usd: 0.0001,
        },
      };
    }
  }
}

// -----------------------------------------------------------------------------
// In-Memory Fake: ToolProvider
// -----------------------------------------------------------------------------

export class FakeToolProvider implements ToolProvider {
  private tools: Map<string, ToolDescriptor> = new Map();
  private handlers: Map<
    string,
    (args: Record<string, unknown>) => Promise<ToolResult> | ToolResult
  > = new Map();
  private invocationLog: Array<{ name: string; args: Record<string, unknown> }> = [];

  constructor() {
    this.registerTool(
      {
        name: "scene.get_tree",
        description: "Get scene hierarchy",
        input_schema: { type: "object" },
        category: "read",
        risk_level: "safe",
      },
      async () => ({ content: { root_entities: [] }, is_error: false })
    );

    this.registerTool(
      {
        name: "script.write",
        description: "Write script file",
        input_schema: { type: "object", required: ["path", "content"] },
        category: "write",
        risk_level: "medium",
      },
      async (args) => ({ content: { success: true, path: args.path }, is_error: false })
    );
  }

  public registerTool(
    descriptor: ToolDescriptor,
    handler: (args: Record<string, unknown>) => Promise<ToolResult> | ToolResult
  ): void {
    this.tools.set(descriptor.name, descriptor);
    this.handlers.set(descriptor.name, handler);
  }

  public async listTools(): Promise<ToolDescriptor[]> {
    return Array.from(this.tools.values());
  }

  public async executeTool(
    name: string,
    args: Record<string, unknown>
  ): Promise<ToolResult> {
    this.invocationLog.push({ name, args });
    const handler = this.handlers.get(name);
    if (!handler) {
      return {
        content: `Tool '${name}' not found`,
        is_error: true,
      };
    }
    return await handler(args);
  }

  public getInvocations() {
    return this.invocationLog;
  }
}

// -----------------------------------------------------------------------------
// In-Memory Fake: SafetyPolicy & CheckpointManager
// -----------------------------------------------------------------------------

export class FakeSafetyPolicy implements SafetyPolicy {
  public async evaluate(
    tool: ToolDescriptor,
    _args: Record<string, unknown>,
    settings: AgentSettings
  ): Promise<SafetyCheckResult> {
    if (settings.permission_mode === "autonomous") {
      return {
        allowed: true,
        requires_permission: false,
        risk_level: tool.risk_level,
      };
    }

    if (settings.permission_mode === "ask_all") {
      return {
        allowed: true,
        requires_permission: true,
        risk_level: tool.risk_level,
        reason: "Settings require permission for all tool calls",
      };
    }

    // ask_risky
    const isRisky = ["medium", "high", "critical"].includes(tool.risk_level);
    return {
      allowed: true,
      requires_permission: isRisky,
      risk_level: tool.risk_level,
      reason: isRisky ? `Tool '${tool.name}' has risk level '${tool.risk_level}'` : undefined,
    };
  }
}

export class FakeCheckpointManager implements CheckpointManager {
  private checkpoints: Map<string, Checkpoint> = new Map();
  private nextId = 1;

  public async createCheckpoint(
    runId: string,
    description: string,
    affectedFiles: string[] = []
  ): Promise<Checkpoint> {
    const id = `chk_${this.nextId++}`;
    const checkpoint: Checkpoint = {
      id,
      run_id: runId,
      timestamp: Date.now(),
      git_ref: `refs/checkpoints/${id}`,
      description,
      affected_files: affectedFiles,
    };
    this.checkpoints.set(id, checkpoint);
    return checkpoint;
  }

  public async revertToCheckpoint(checkpointId: string): Promise<boolean> {
    return this.checkpoints.has(checkpointId);
  }

  public async listCheckpoints(runId: string): Promise<Checkpoint[]> {
    return Array.from(this.checkpoints.values()).filter((c) => c.run_id === runId);
  }
}

// -----------------------------------------------------------------------------
// In-Memory Fake: Orchestrator
// -----------------------------------------------------------------------------

export class FakeOrchestrator implements Orchestrator {
  private listeners: Set<(event: AgentEvent) => void> = new Set();
  private summaries: Map<string, RunSummary> = new Map();
  private currentPlan: Plan | null = null;
  private nextRunId = 1;

  constructor(
    private llm: LlmProvider,
    private tools: ToolProvider,
    private safety: SafetyPolicy,
    private checkpoints: CheckpointManager,
    private settings: AgentSettings
  ) {}

  public subscribeEvents(listener: (event: AgentEvent) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private emitEvent(event: AgentEvent) {
    for (const listener of this.listeners) {
      try {
        listener(event);
      } catch {
        // ignore listener errors
      }
    }
  }

  public async startRun(userPrompt: string, customRunId?: string): Promise<string> {
    const runId = customRunId || `run_${this.nextRunId++}`;
    const now = Date.now();

    const summary: RunSummary = {
      run_id: runId,
      status: "running",
      start_time: now,
      total_turns: 0,
      budget_used: {
        input_tokens: 0,
        output_tokens: 0,
        total_tokens: 0,
        estimated_cost_usd: 0,
      },
    };
    this.summaries.set(runId, summary);

    this.emitEvent({
      type: "run_started",
      run_id: runId,
      timestamp: now,
      data: { prompt: userPrompt },
    });

    const checkpoint = await this.checkpoints.createCheckpoint(
      runId,
      `Initial checkpoint for: ${userPrompt}`
    );
    this.emitEvent({
      type: "checkpoint_created",
      run_id: runId,
      timestamp: Date.now(),
      data: { checkpoint },
    });

    // Generate initial plan
    this.currentPlan = {
      id: `plan_${runId}`,
      title: `Plan for: ${userPrompt}`,
      steps: [
        {
          id: "step_1",
          description: "Analyze scene and current project state",
          status: "in_progress",
          acceptance_checks: ["Scene hierarchy inspected"],
          tool_call_ids: [],
        },
      ],
    };
    this.emitEvent({
      type: "plan_updated",
      run_id: runId,
      timestamp: Date.now(),
      data: { plan: this.currentPlan },
    });

    // Stream first LLM turn
    const chunks = this.llm.streamTurn([{ role: "user", content: userPrompt }]);
    for await (const chunk of chunks) {
      if (chunk.delta) {
        this.emitEvent({
          type: "message_delta",
          run_id: runId,
          timestamp: Date.now(),
          data: { delta: chunk.delta },
        });
      }
      if (chunk.tool_calls) {
        for (const tc of chunk.tool_calls) {
          this.emitEvent({
            type: "tool_call_started",
            run_id: runId,
            timestamp: Date.now(),
            data: { tool_call: tc },
          });

          const result = await this.tools.executeTool(tc.name, tc.arguments);
          this.emitEvent({
            type: "tool_call_finished",
            run_id: runId,
            timestamp: Date.now(),
            data: { tool_call_id: tc.id, result },
          });
        }
      }
      if (chunk.usage) {
        summary.budget_used = chunk.usage;
        this.emitEvent({
          type: "budget_update",
          run_id: runId,
          timestamp: Date.now(),
          data: { usage: summary.budget_used },
        });
      }
    }

    this.emitEvent({
      type: "message_done",
      run_id: runId,
      timestamp: Date.now(),
      data: {},
    });

    summary.status = "completed";
    summary.end_time = Date.now();
    summary.total_turns = 1;

    this.emitEvent({
      type: "run_finished",
      run_id: runId,
      timestamp: summary.end_time,
      data: { summary },
    });

    return runId;
  }

  public async dispatchCommand(command: AgentCommand): Promise<void> {
    if (command.command === "revert_checkpoint" && command.payload.checkpoint_id) {
      await this.checkpoints.revertToCheckpoint(String(command.payload.checkpoint_id));
    }
  }

  public async getSummary(runId: string): Promise<RunSummary | null> {
    return this.summaries.get(runId) || null;
  }

  public async getCurrentPlan(): Promise<Plan | null> {
    return this.currentPlan;
  }
}

// -----------------------------------------------------------------------------
// In-Memory Fake: SkillProvider, ProjectBrain, DocsProvider
// -----------------------------------------------------------------------------

export class FakeSkillProvider implements SkillProvider {
  private skills: Map<string, SkillDefinition> = new Map([
    [
      "platformer-builder",
      {
        name: "platformer-builder",
        description: "Creates 2D platformer levels, hazards, and coin entities",
        instructions: "Use tilemap.fill_rect and scene.create_entity with Sprite and Collider2D.",
      },
    ],
  ]);

  public async listSkills(): Promise<SkillDefinition[]> {
    return Array.from(this.skills.values());
  }

  public async getSkill(name: string): Promise<SkillDefinition | null> {
    return this.skills.get(name) || null;
  }
}

export class FakeProjectBrain implements ProjectBrain {
  private knowledge: Map<string, string> = new Map();

  public async recordKnowledge(topic: string, content: string): Promise<void> {
    this.knowledge.set(topic, content);
  }

  public async queryKnowledge(query: string): Promise<string[]> {
    const results: string[] = [];
    for (const [topic, content] of this.knowledge.entries()) {
      if (topic.includes(query) || content.includes(query)) {
        results.push(`[${topic}]: ${content}`);
      }
    }
    return results;
  }

  public async getContextSnapshot(): Promise<Record<string, unknown>> {
    return Object.fromEntries(this.knowledge.entries());
  }
}

export class FakeDocsProvider implements DocsProvider {
  private docs: Map<string, string> = new Map([
    ["docs/ENGINE_STATE.md", "# Forge Engine State Audit\nDetailed architecture and state."],
    ["docs/agent-guides/agent.md", "# Agent Guide\nCassette specification and fakes."],
  ]);

  public async getDoc(path: string): Promise<string | null> {
    return this.docs.get(path) || null;
  }

  public async searchDocs(query: string): Promise<Array<{ path: string; snippet: string }>> {
    const results: Array<{ path: string; snippet: string }> = [];
    for (const [path, content] of this.docs.entries()) {
      if (content.toLowerCase().includes(query.toLowerCase())) {
        results.push({ path, snippet: content.slice(0, 100) });
      }
    }
    return results;
  }
}
