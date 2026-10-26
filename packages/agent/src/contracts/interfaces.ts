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

// -----------------------------------------------------------------------------
// LLM Provider Interface
// -----------------------------------------------------------------------------

export interface LlmMessage {
  role: "system" | "user" | "assistant" | "tool";
  content: string;
  name?: string;
  tool_call_id?: string;
}

export interface LlmToolCall {
  id: string;
  name: string;
  arguments: Record<string, unknown>;
}

export interface LlmTurnChunk {
  delta?: string;
  thought?: string;
  tool_calls?: LlmToolCall[];
  finish_reason?: "stop" | "tool_calls" | "length" | "error";
  usage?: BudgetUsage;
}

export interface LlmTurnOptions {
  model?: string;
  temperature?: number;
  max_output_tokens?: number;
  tools?: ToolDescriptor[];
  system_instruction?: string;
  signal?: AbortSignal;
}

export interface LlmProvider {
  streamTurn(
    messages: LlmMessage[],
    options?: LlmTurnOptions
  ): AsyncIterable<LlmTurnChunk>;
  getModelName(): string;
}

// -----------------------------------------------------------------------------
// Tool Provider Interface
// -----------------------------------------------------------------------------

export interface ToolResult {
  content: unknown;
  is_error: boolean;
}

export interface ToolProvider {
  listTools(): Promise<ToolDescriptor[]>;
  executeTool(
    name: string,
    args: Record<string, unknown>,
    signal?: AbortSignal
  ): Promise<ToolResult>;
}

// -----------------------------------------------------------------------------
// Safety Policy & Checkpoint Manager
// -----------------------------------------------------------------------------

export interface SafetyCheckResult {
  allowed: boolean;
  requires_permission: boolean;
  risk_level: "safe" | "low" | "medium" | "high" | "critical";
  reason?: string;
}

export interface SafetyPolicy {
  evaluate(
    tool: ToolDescriptor,
    args: Record<string, unknown>,
    settings: AgentSettings
  ): Promise<SafetyCheckResult>;
}

export interface CheckpointManager {
  createCheckpoint(
    runId: string,
    description: string,
    affectedFiles?: string[]
  ): Promise<Checkpoint>;
  revertToCheckpoint(checkpointId: string): Promise<boolean>;
  listCheckpoints(runId: string): Promise<Checkpoint[]>;
}

// -----------------------------------------------------------------------------
// Orchestrator Interface
// -----------------------------------------------------------------------------

export interface OrchestratorOptions {
  settings: AgentSettings;
  llm: LlmProvider;
  tools: ToolProvider;
  safety: SafetyPolicy;
  checkpoints: CheckpointManager;
}

export interface Orchestrator {
  startRun(userPrompt: string, runId?: string): Promise<string>;
  dispatchCommand(command: AgentCommand): Promise<void>;
  subscribeEvents(listener: (event: AgentEvent) => void): () => void;
  getSummary(runId: string): Promise<RunSummary | null>;
  getCurrentPlan(): Promise<Plan | null>;
}

// -----------------------------------------------------------------------------
// Skill Provider (Stub)
// -----------------------------------------------------------------------------

export interface SkillDefinition {
  name: string;
  description: string;
  instructions: string;
}

export interface SkillProvider {
  listSkills(): Promise<SkillDefinition[]>;
  getSkill(name: string): Promise<SkillDefinition | null>;
}

// -----------------------------------------------------------------------------
// Project Brain & Docs Provider
// -----------------------------------------------------------------------------

export interface ProjectBrain {
  recordKnowledge(topic: string, content: string): Promise<void>;
  queryKnowledge(query: string): Promise<string[]>;
  getContextSnapshot(): Promise<Record<string, unknown>>;
}

export interface DocsProvider {
  getDoc(path: string): Promise<string | null>;
  searchDocs(query: string): Promise<Array<{ path: string; snippet: string }>>;
}
