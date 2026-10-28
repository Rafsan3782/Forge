import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { EvalSuiteReport } from "./types.ts";

interface ToolCallData {
  tool_call?: {
    name?: string;
  };
}

export function generateHtmlReport(report: EvalSuiteReport): string {
  const passRate = report.totalTasks > 0
    ? Math.round((report.passedTasks / report.totalTasks) * 100)
    : 0;

  const statusColor = report.failedTasks === 0 ? "#10b981" : "#ef4444";

  const taskCards = report.results
    .map((task) => {
      const taskBadgeClass = task.passed ? "badge-pass" : "badge-fail";
      const checkItems = task.checks
        .map(
          (c) => `
        <li class="check-item ${c.passed ? "check-pass" : "check-fail"}">
          <span class="check-icon">${c.passed ? "✔" : "✖"}</span>
          <span class="check-name">${escapeHtml(c.name)}</span>
          ${c.message ? `<div class="check-msg">${escapeHtml(c.message)}</div>` : ""}
        </li>
      `
        )
        .join("");

      const timelineItems = task.events
        .map(
          (ev) => `
        <div class="timeline-event">
          <span class="timeline-time">+${ev.timestamp % 100000}ms</span>
          <span class="timeline-type">${escapeHtml(ev.type)}</span>
          ${
            ev.type === "tool_call_started"
              ? `<span class="timeline-detail">${escapeHtml(
                  (ev.data as ToolCallData)?.tool_call?.name || ""
                )}</span>`
              : ""
          }
        </div>
      `
        )
        .join("");

      const screenshotsHtml = (task.finalScreenshots || [])
        .map(
          (src, idx) => `
        <div class="screenshot-box">
          <img src="${src}" alt="Screenshot ${idx + 1}" />
          <div class="screenshot-caption">Screenshot ${idx + 1}</div>
        </div>
      `
        )
        .join("");

      return `
      <div class="task-card ${task.passed ? "task-pass" : "task-fail"}">
        <div class="task-header">
          <div>
            <h3 class="task-title">${escapeHtml(task.taskName)}</h3>
            <span class="task-id">ID: ${escapeHtml(task.taskId)}</span>
          </div>
          <div class="task-metrics">
            <span class="metric-tag">Score: ${(task.score * 100).toFixed(0)}%</span>
            <span class="metric-tag">Turns: ${task.turns}</span>
            <span class="metric-tag">Tokens: ${task.tokens}</span>
            <span class="metric-tag">Cost: $${task.costUsd.toFixed(4)}</span>
            <span class="badge ${taskBadgeClass}">${task.passed ? "PASSED" : "FAILED"}</span>
          </div>
        </div>

        <div class="task-body">
          <div class="task-section">
            <h4>Checks (${task.checks.filter((c) => c.passed).length}/${task.checks.length})</h4>
            <ul class="check-list">
              ${checkItems}
            </ul>
          </div>

          ${
            screenshotsHtml
              ? `
          <div class="task-section">
            <h4>Screenshots</h4>
            <div class="screenshots-container">
              ${screenshotsHtml}
            </div>
          </div>
          `
              : ""
          }

          <div class="task-section">
            <details>
              <summary>Event Timeline (${task.events.length} events)</summary>
              <div class="timeline-list">
                ${timelineItems || "<p>No events recorded</p>"}
              </div>
            </details>
          </div>
        </div>
      </div>
    `;
    })
    .join("");

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Forge Agent Eval Report</title>
  <style>
    :root {
      --bg: #090a0f;
      --card-bg: #131722;
      --border: #23283b;
      --text: #f1f5f9;
      --text-muted: #94a3b8;
      --pass: #10b981;
      --fail: #ef4444;
      --accent: #6366f1;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      background: var(--bg);
      color: var(--text);
      padding: 32px;
      line-height: 1.5;
    }
    .container { max-width: 1200px; margin: 0 auto; }
    header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 24px;
      border-bottom: 1px solid var(--border);
      padding-bottom: 20px;
    }
    h1 { font-size: 24px; font-weight: 700; }
    .meta-subtitle { color: var(--text-muted); font-size: 14px; margin-top: 4px; }
    .stats-row {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
      gap: 16px;
      margin-bottom: 32px;
    }
    .stat-card {
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 16px;
    }
    .stat-label { font-size: 13px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em; }
    .stat-value { font-size: 24px; font-weight: 700; margin-top: 6px; }
    .task-card {
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 8px;
      margin-bottom: 20px;
      overflow: hidden;
    }
    .task-card.task-pass { border-left: 4px solid var(--pass); }
    .task-card.task-fail { border-left: 4px solid var(--fail); }
    .task-header {
      padding: 16px 20px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      background: rgba(255, 255, 255, 0.02);
      border-bottom: 1px solid var(--border);
    }
    .task-title { font-size: 18px; font-weight: 600; }
    .task-id { font-size: 12px; color: var(--text-muted); }
    .task-metrics { display: flex; align-items: center; gap: 10px; }
    .metric-tag {
      font-size: 12px;
      background: #1e2436;
      padding: 4px 8px;
      border-radius: 4px;
      color: var(--text-muted);
    }
    .badge {
      font-size: 12px;
      font-weight: 600;
      padding: 4px 10px;
      border-radius: 12px;
    }
    .badge-pass { background: rgba(16, 185, 129, 0.2); color: var(--pass); }
    .badge-fail { background: rgba(239, 68, 68, 0.2); color: var(--fail); }
    .task-body { padding: 20px; }
    .task-section { margin-bottom: 16px; }
    .task-section h4 { font-size: 14px; font-weight: 600; margin-bottom: 8px; color: #cbd5e1; }
    .check-list { list-style: none; display: flex; flex-direction: column; gap: 8px; }
    .check-item {
      display: flex;
      align-items: flex-start;
      gap: 10px;
      padding: 8px 12px;
      border-radius: 6px;
      background: #0f131d;
      font-size: 14px;
    }
    .check-pass .check-icon { color: var(--pass); font-weight: bold; }
    .check-fail .check-icon { color: var(--fail); font-weight: bold; }
    .check-msg { font-size: 12px; color: var(--text-muted); margin-top: 4px; }
    .screenshots-container { display: flex; gap: 16px; flex-wrap: wrap; margin-top: 8px; }
    .screenshot-box img { max-width: 280px; border-radius: 6px; border: 1px solid var(--border); }
    .screenshot-caption { font-size: 11px; color: var(--text-muted); text-align: center; margin-top: 4px; }
    details summary { cursor: pointer; color: var(--accent); font-size: 13px; font-weight: 500; }
    .timeline-list { max-height: 200px; overflow-y: auto; background: #0c0e14; padding: 12px; border-radius: 6px; margin-top: 8px; }
    .timeline-event { font-family: monospace; font-size: 12px; margin-bottom: 4px; display: flex; gap: 12px; }
    .timeline-time { color: var(--text-muted); }
    .timeline-type { color: #818cf8; }
    .timeline-detail { color: #38bdf8; }
  </style>
</head>
<body>
  <div class="container">
    <header>
      <div>
        <h1>Forge Agent Evaluation Report</h1>
        <div class="meta-subtitle">Mode: ${report.mode.toUpperCase()} | Generated: ${report.timestamp}</div>
      </div>
      <div>
        <span class="badge ${report.failedTasks === 0 ? "badge-pass" : "badge-fail"}" style="font-size: 16px; padding: 6px 16px;">
          ${report.failedTasks === 0 ? "ALL PASS" : `${report.failedTasks} FAILED`}
        </span>
      </div>
    </header>

    <div class="stats-row">
      <div class="stat-card">
        <div class="stat-label">Pass Rate</div>
        <div class="stat-value" style="color: ${statusColor}">${passRate}%</div>
      </div>
      <div class="stat-card">
        <div class="stat-label">Tasks Passed / Total</div>
        <div class="stat-value">${report.passedTasks} / ${report.totalTasks}</div>
      </div>
      <div class="stat-card">
        <div class="stat-label">Total Cost</div>
        <div class="stat-value">$${report.totalCostUsd.toFixed(4)}</div>
      </div>
      <div class="stat-card">
        <div class="stat-label">Total Tokens</div>
        <div class="stat-value">${report.totalTokens}</div>
      </div>
      <div class="stat-card">
        <div class="stat-label">Total Duration</div>
        <div class="stat-value">${(report.totalDurationMs / 1000).toFixed(1)}s</div>
      </div>
    </div>

    <div class="tasks-container">
      ${taskCards}
    </div>
  </div>
</body>
</html>`;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

export async function saveReports(
  report: EvalSuiteReport,
  outputDir: string
): Promise<{ jsonPath: string; htmlPath: string }> {
  await fs.mkdir(outputDir, { recursive: true });
  const jsonPath = path.join(outputDir, "eval_report.json");
  const htmlPath = path.join(outputDir, "eval_report.html");

  await fs.writeFile(jsonPath, JSON.stringify(report, null, 2), "utf-8");
  await fs.writeFile(htmlPath, generateHtmlReport(report), "utf-8");

  return { jsonPath, htmlPath };
}
