import type { AgentEvent, RunSummary } from "@forge/protocol";

export interface EvalTaskLimits {
  timeout_seconds?: number;
  max_turns?: number;
  max_tokens?: number;
  max_cost_usd?: number;
}

export interface EvalTaskConfig {
  id: string;
  name: string;
  description: string;
  prompt: string;
  template: string;
  limits: EvalTaskLimits;
  tags: string[];
}

export interface CheckContext {
  taskDir: string;
  workspaceDir: string;
  templateDir: string;
  events: AgentEvent[];
  summary: RunSummary | null;
  toolInvocations: Array<{ name: string; args: Record<string, unknown> }>;
}

export interface CheckResult {
  name: string;
  passed: boolean;
  message?: string;
  is_soft?: boolean;
}

export type CheckFn = (ctx: CheckContext) => Promise<CheckResult> | CheckResult;

export interface CassetteTurnInteraction {
  turn_id: string;
  request: {
    messages: Array<{
      role: "system" | "user" | "assistant" | "tool";
      content: string;
      name?: string;
      tool_call_id?: string;
    }>;
    available_tools?: string[];
  };
  response: {
    thought?: string;
    tool_calls?: Array<{
      id: string;
      name: string;
      arguments: Record<string, unknown>;
    }>;
    message_delta?: string;
    finish_reason?: "stop" | "tool_calls" | "length" | "error";
    usage?: {
      input_tokens: number;
      output_tokens: number;
      total_tokens: number;
      estimated_cost_usd: number;
    };
  };
}

export interface CassetteData {
  $schema?: string;
  version: number;
  metadata: {
    recording_id: string;
    created_at: string;
    model: string;
    temperature?: number;
  };
  interactions: CassetteTurnInteraction[];
}

export interface TaskEvalResult {
  taskId: string;
  taskName: string;
  passed: boolean;
  score: number; // 0 to 1
  checks: CheckResult[];
  durationMs: number;
  turns: number;
  tokens: number;
  costUsd: number;
  checkpointsCreated: number;
  events: AgentEvent[];
  toolCalls: Array<{ name: string; args: Record<string, unknown> }>;
  finalScreenshots: string[];
  error?: string;
}

export interface EvalSuiteReport {
  timestamp: string;
  mode: "replay" | "live";
  gitCommit?: string;
  gitBranch?: string;
  totalTasks: number;
  passedTasks: number;
  failedTasks: number;
  totalCostUsd: number;
  totalTokens: number;
  totalDurationMs: number;
  results: TaskEvalResult[];
}

export interface EvalDiffRegression {
  taskId: string;
  checkName: string;
  type: "failed_regression" | "cost_regression" | "step_regression";
  previous: string | number;
  current: string | number;
}

export interface EvalSuiteDiff {
  previousTimestamp: string;
  currentTimestamp: string;
  regressions: EvalDiffRegression[];
  improvements: Array<{ taskId: string; message: string }>;
  costDeltaUsd: number;
}
