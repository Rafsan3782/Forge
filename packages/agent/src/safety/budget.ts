import type { AgentEvent } from "@forge/protocol";

export interface BudgetConfig {
  max_tokens?: number;
  max_cost_usd?: number;
  max_wall_time_ms?: number;
  max_steps?: number;
  max_tool_calls?: number;
  max_consecutive_errors?: number;
  warning_threshold?: number; // e.g. 0.8 for 80%
}

export interface BudgetState {
  tokens_used: number;
  cost_usd: number;
  wall_time_ms: number;
  steps_taken: number;
  tool_calls_made: number;
  consecutive_errors: number;
}

export interface BudgetCheckResult {
  exceeded: boolean;
  near_limit: boolean;
  reason?: string;
  exceeded_dimension?: keyof BudgetState;
}

export class BudgetTracker {
  private config: Required<BudgetConfig>;
  private state: BudgetState;
  private startTime: number;
  private warnedDimensions: Set<string> = new Set();

  constructor(config: BudgetConfig = {}, initialState?: Partial<BudgetState>) {
    this.config = {
      max_tokens: config.max_tokens ?? Infinity,
      max_cost_usd: config.max_cost_usd ?? Infinity,
      max_wall_time_ms: config.max_wall_time_ms ?? Infinity,
      max_steps: config.max_steps ?? Infinity,
      max_tool_calls: config.max_tool_calls ?? Infinity,
      max_consecutive_errors: config.max_consecutive_errors ?? 5,
      warning_threshold: config.warning_threshold ?? 0.8,
    };

    this.startTime = Date.now();
    this.state = {
      tokens_used: initialState?.tokens_used ?? 0,
      cost_usd: initialState?.cost_usd ?? 0,
      wall_time_ms: initialState?.wall_time_ms ?? 0,
      steps_taken: initialState?.steps_taken ?? 0,
      tool_calls_made: initialState?.tool_calls_made ?? 0,
      consecutive_errors: initialState?.consecutive_errors ?? 0,
    };
  }

  public getState(): Readonly<BudgetState> {
    this.updateWallTime();
    return { ...this.state };
  }

  public getConfig(): Readonly<Required<BudgetConfig>> {
    return { ...this.config };
  }

  public addUsage(delta: {
    tokens?: number;
    cost_usd?: number;
    steps?: number;
    tool_calls?: number;
    is_error?: boolean;
  }): void {
    if (delta.tokens) {
      this.state.tokens_used += delta.tokens;
    }
    if (delta.cost_usd) {
      this.state.cost_usd += delta.cost_usd;
    }
    if (delta.steps) {
      this.state.steps_taken += delta.steps;
    }
    if (delta.tool_calls) {
      this.state.tool_calls_made += delta.tool_calls;
    }
    if (delta.is_error !== undefined) {
      if (delta.is_error) {
        this.state.consecutive_errors += 1;
      } else {
        this.state.consecutive_errors = 0;
      }
    }
    this.updateWallTime();
  }

  public extendBudget(additional: Partial<BudgetConfig>): void {
    if (additional.max_tokens !== undefined) {
      this.config.max_tokens = Math.max(this.state.tokens_used, this.config.max_tokens) + additional.max_tokens;
    }
    if (additional.max_cost_usd !== undefined) {
      this.config.max_cost_usd = Math.max(this.state.cost_usd, this.config.max_cost_usd) + additional.max_cost_usd;
    }
    if (additional.max_wall_time_ms !== undefined) {
      this.config.max_wall_time_ms = Math.max(this.state.wall_time_ms, this.config.max_wall_time_ms) + additional.max_wall_time_ms;
    }
    if (additional.max_steps !== undefined) {
      this.config.max_steps = Math.max(this.state.steps_taken, this.config.max_steps) + additional.max_steps;
    }
    if (additional.max_tool_calls !== undefined) {
      this.config.max_tool_calls = Math.max(this.state.tool_calls_made, this.config.max_tool_calls) + additional.max_tool_calls;
    }
    if (additional.max_consecutive_errors !== undefined) {
      this.config.max_consecutive_errors = additional.max_consecutive_errors;
      this.state.consecutive_errors = 0;
    }
    this.warnedDimensions.clear();
  }

  public resetConsecutiveErrors(): void {
    this.state.consecutive_errors = 0;
  }

  private updateWallTime(): void {
    this.state.wall_time_ms = Date.now() - this.startTime;
  }

  public check(): BudgetCheckResult {
    this.updateWallTime();

    const pairs: Array<{
      current: number;
      max: number;
      name: keyof BudgetState;
      label: string;
    }> = [
      { current: this.state.tokens_used, max: this.config.max_tokens, name: "tokens_used", label: "tokens" },
      { current: this.state.cost_usd, max: this.config.max_cost_usd, name: "cost_usd", label: "estimated cost (USD)" },
      { current: this.state.wall_time_ms, max: this.config.max_wall_time_ms, name: "wall_time_ms", label: "wall time (ms)" },
      { current: this.state.steps_taken, max: this.config.max_steps, name: "steps_taken", label: "plan steps" },
      { current: this.state.tool_calls_made, max: this.config.max_tool_calls, name: "tool_calls_made", label: "tool calls" },
      { current: this.state.consecutive_errors, max: this.config.max_consecutive_errors, name: "consecutive_errors", label: "consecutive errors" },
    ];

    for (const item of pairs) {
      if (item.max !== Infinity && item.current >= item.max) {
        return {
          exceeded: true,
          near_limit: true,
          reason: `Budget exceeded: ${item.label} reached ${item.current} (limit: ${item.max})`,
          exceeded_dimension: item.name,
        };
      }
    }

    for (const item of pairs) {
      if (item.max !== Infinity && item.max > 0) {
        const ratio = item.current / item.max;
        if (ratio >= this.config.warning_threshold) {
          const isNewWarning = !this.warnedDimensions.has(item.name);
          if (isNewWarning) {
            this.warnedDimensions.add(item.name);
          }
          return {
            exceeded: false,
            near_limit: true,
            reason: `Budget warning: ${item.label} reached ${Math.round(ratio * 100)}% (${item.current}/${item.max})`,
            exceeded_dimension: item.name,
          };
        }
      }
    }

    return {
      exceeded: false,
      near_limit: false,
    };
  }

  public createBudgetUpdateEvent(runId: string, warning?: string): AgentEvent {
    this.updateWallTime();
    return {
      type: "budget_update",
      run_id: runId,
      timestamp: Date.now(),
      data: {
        budget_state: this.getState(),
        limits: this.getConfig(),
        warning: warning ?? null,
      },
    };
  }
}
