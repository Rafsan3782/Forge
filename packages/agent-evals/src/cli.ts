import * as path from "node:path";
import { EvalRunner } from "./runner.ts";
import { buildSuiteReport, compareRuns } from "./scoring.ts";
import { saveReports } from "./reports.ts";
import { getAllTasks } from "./tasks/index.ts";
import type { EvalSuiteReport } from "./types.ts";

export interface CliOptions {
  mode: "replay" | "live";
  record?: boolean;
  tasks?: string[];
  outputDir?: string;
  concurrency?: number;
  compareWith?: string;
}

export async function runCli(options: CliOptions): Promise<EvalSuiteReport> {
  const runner = new EvalRunner({
    mode: options.mode,
    maxConcurrency: options.concurrency || 4,
    recordCassette: options.record,
  });

  let tasks = getAllTasks();
  if (options.tasks && options.tasks.length > 0) {
    tasks = tasks.filter((t) => options.tasks!.includes(t.config.id));
  }

  console.log(`Starting Forge Agent Evals (${options.mode.toUpperCase()} mode, ${tasks.length} tasks)...`);
  const results = await runner.runSuite(tasks);

  const report = buildSuiteReport(results, options.mode);
  const outDir = options.outputDir || path.resolve(process.cwd(), "eval-reports");
  const { jsonPath, htmlPath } = await saveReports(report, outDir);

  console.log(`\nEvaluation Completed:`);
  console.log(`  Passed: ${report.passedTasks}/${report.totalTasks}`);
  console.log(`  Total Tokens: ${report.totalTokens}`);
  console.log(`  Total Cost: $${report.totalCostUsd.toFixed(4)}`);
  console.log(`  HTML Report: ${htmlPath}`);
  console.log(`  JSON Report: ${jsonPath}`);

  if (options.compareWith) {
    try {
      const fs = await import("node:fs/promises");
      const baseRaw = await fs.readFile(options.compareWith, "utf-8");
      const baseReport = JSON.parse(baseRaw) as EvalSuiteReport;
      const diff = compareRuns(baseReport, report);

      console.log(`\nRun Comparison:`);
      console.log(`  Cost Delta: $${diff.costDeltaUsd.toFixed(4)}`);
      if (diff.regressions.length > 0) {
        console.error(`  ⚠️ Regressions detected: ${diff.regressions.length}`);
        for (const reg of diff.regressions) {
          console.error(`    - [${reg.type}] Task ${reg.taskId}: ${reg.checkName} (was ${reg.previous}, now ${reg.current})`);
        }
      } else {
        console.log(`  No regressions detected.`);
      }
    } catch (e: unknown) {
      console.warn(`Could not perform run comparison: ${e instanceof Error ? e.message : String(e)}`);
    }
  }

  return report;
}

// When executed directly via node
if (process.argv[1]?.includes("cli.ts") || process.argv[1]?.includes("cli.js")) {
  const modeArg = process.argv.includes("--live") ? "live" : "replay";
  const recordArg = process.argv.includes("--record");
  runCli({ mode: modeArg, record: recordArg }).then((report) => {
    if (report.failedTasks > 0) {
      process.exit(1);
    }
  }).catch((err) => {
    console.error("Eval run failed:", err);
    process.exit(1);
  });
}
