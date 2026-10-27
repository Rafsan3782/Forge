import type {
  TaskEvalResult,
  EvalSuiteReport,
  EvalSuiteDiff,
  EvalDiffRegression,
} from "./types.ts";

export interface ScoreWeights {
  checksWeight?: number;
  stepEfficiencyWeight?: number;
  costEfficiencyWeight?: number;
}

export function computeTaskWeightedScore(
  result: TaskEvalResult,
  weights: ScoreWeights = {}
): number {
  const wChecks = weights.checksWeight ?? 0.7;
  const wSteps = weights.stepEfficiencyWeight ?? 0.15;
  const wCost = weights.costEfficiencyWeight ?? 0.15;

  const passedChecks = result.checks.filter((c) => c.passed).length;
  const totalChecks = result.checks.length || 1;
  const checkScore = passedChecks / totalChecks;

  // Step efficiency: <= 5 steps is 1.0, 20 steps is 0.0
  const stepScore = Math.max(0, Math.min(1, 1 - (result.turns - 1) / 19));

  // Cost efficiency: <= $0.01 is 1.0, >= $0.50 is 0.0
  const costScore = Math.max(0, Math.min(1, 1 - result.costUsd / 0.5));

  const weighted = checkScore * wChecks + stepScore * wSteps + costScore * wCost;
  return Number(weighted.toFixed(3));
}

export function buildSuiteReport(
  results: TaskEvalResult[],
  mode: "replay" | "live",
  meta: { gitCommit?: string; gitBranch?: string } = {}
): EvalSuiteReport {
  const passedCount = results.filter((r) => r.passed).length;
  const failedCount = results.length - passedCount;
  const totalCost = results.reduce((sum, r) => sum + r.costUsd, 0);
  const totalTokens = results.reduce((sum, r) => sum + r.tokens, 0);
  const totalDuration = results.reduce((sum, r) => sum + r.durationMs, 0);

  return {
    timestamp: new Date().toISOString(),
    mode,
    gitCommit: meta.gitCommit,
    gitBranch: meta.gitBranch,
    totalTasks: results.length,
    passedTasks: passedCount,
    failedTasks: failedCount,
    totalCostUsd: Number(totalCost.toFixed(4)),
    totalTokens,
    totalDurationMs: totalDuration,
    results,
  };
}

export function compareRuns(
  baseline: EvalSuiteReport,
  candidate: EvalSuiteReport,
  tolerances: { costIncreaseToleranceUsd?: number; stepIncreaseTolerance?: number } = {}
): EvalSuiteDiff {
  const regressions: EvalDiffRegression[] = [];
  const improvements: Array<{ taskId: string; message: string }> = [];

  const costTolerance = tolerances.costIncreaseToleranceUsd ?? 0.02;
  const stepTolerance = tolerances.stepIncreaseTolerance ?? 2;

  const baseMap = new Map(baseline.results.map((r) => [r.taskId, r]));

  for (const curr of candidate.results) {
    const base = baseMap.get(curr.taskId);
    if (!base) continue;

    // Check pass/fail regression
    if (base.passed && !curr.passed) {
      regressions.push({
        taskId: curr.taskId,
        checkName: "Overall task pass",
        type: "failed_regression",
        previous: "PASSED",
        current: "FAILED",
      });
    } else if (!base.passed && curr.passed) {
      improvements.push({
        taskId: curr.taskId,
        message: "Task previously failed but is now passing!",
      });
    }

    // Individual check regressions
    const baseChecks = new Map(base.checks.map((c) => [c.name, c.passed]));
    for (const check of curr.checks) {
      const baseCheckPassed = baseChecks.get(check.name);
      if (baseCheckPassed === true && !check.passed) {
        regressions.push({
          taskId: curr.taskId,
          checkName: check.name,
          type: "failed_regression",
          previous: "PASSED",
          current: "FAILED",
        });
      }
    }

    // Step efficiency regression
    if (curr.turns > base.turns + stepTolerance) {
      regressions.push({
        taskId: curr.taskId,
        checkName: "Step Efficiency",
        type: "step_regression",
        previous: base.turns,
        current: curr.turns,
      });
    }

    // Cost regression
    if (curr.costUsd > base.costUsd + costTolerance) {
      regressions.push({
        taskId: curr.taskId,
        checkName: "Cost Efficiency",
        type: "cost_regression",
        previous: `$${base.costUsd.toFixed(4)}`,
        current: `$${curr.costUsd.toFixed(4)}`,
      });
    }
  }

  return {
    previousTimestamp: baseline.timestamp,
    currentTimestamp: candidate.timestamp,
    regressions,
    improvements,
    costDeltaUsd: Number((candidate.totalCostUsd - baseline.totalCostUsd).toFixed(4)),
  };
}
