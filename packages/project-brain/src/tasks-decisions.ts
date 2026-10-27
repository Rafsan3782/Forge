import type { TaskItem, DecisionItem } from "./types.ts";

export function parseTasks(content: string): TaskItem[] {
  const tasks: TaskItem[] = [];
  const lines = content.split(/\r?\n/);

  let currentTask: TaskItem | null = null;

  for (const line of lines) {
    const taskHeaderMatch = line.match(/^[-*]\s*\[([ xX~])\]\s*([a-zA-Z0-9_-]+)?(?::\s*|\s+-?\s*)(.*)$/);

    if (taskHeaderMatch) {
      if (currentTask) {
        tasks.push(currentTask);
      }

      const check = taskHeaderMatch[1];
      const parsedId = taskHeaderMatch[2];
      const title = taskHeaderMatch[3].trim();

      let status: TaskItem["status"] = "todo";
      if (check === "x" || check === "X") {
        status = "done";
      } else if (check === "~") {
        status = "in_progress";
      }

      const id = parsedId || `TASK-${String(tasks.length + 1).padStart(3, "0")}`;

      currentTask = {
        id,
        title,
        status,
        acceptance_checks: [],
      };
      continue;
    }

    if (currentTask) {
      const trimmed = line.trim();
      const statusMatch = trimmed.match(/^[-*]\s*(?:status|Status):\s*([a-zA-Z_-]+)/i);
      if (statusMatch) {
        const s = statusMatch[1].toLowerCase();
        if (s === "done" || s === "completed") currentTask.status = "done";
        else if (s === "in_progress" || s === "progress") currentTask.status = "in_progress";
        else if (s === "blocked") currentTask.status = "blocked";
        else currentTask.status = "todo";
        continue;
      }

      const acceptMatch = trimmed.match(/^[-*]\s*(?:acceptance|check|Acceptance):\s*(.*)$/i);
      if (acceptMatch) {
        currentTask.acceptance_checks.push(acceptMatch[1].trim());
        continue;
      }

      if (trimmed.startsWith("- ") || trimmed.startsWith("* ")) {
        currentTask.acceptance_checks.push(trimmed.slice(2).trim());
      }
    }
  }

  if (currentTask) {
    tasks.push(currentTask);
  }

  return tasks;
}

export function serializeTasks(tasks: TaskItem[]): string {
  const lines: string[] = ["# Tasks\n"];

  for (const task of tasks) {
    let check = " ";
    if (task.status === "done") check = "x";
    else if (task.status === "in_progress") check = "~";

    lines.push(`- [${check}] ${task.id}: ${task.title}`);
    lines.push(`  - Status: ${task.status}`);
    for (const checkItem of task.acceptance_checks) {
      lines.push(`  - Acceptance: ${checkItem}`);
    }
    if (task.notes) {
      lines.push(`  - Notes: ${task.notes}`);
    }
    lines.push("");
  }

  return lines.join("\n").trimEnd() + "\n";
}

export function parseDecisions(content: string): DecisionItem[] {
  const decisions: DecisionItem[] = [];
  const lines = content.split(/\r?\n/);

  let currentDate = "";
  let currentDecision = "";
  let currentWhy = "";

  for (const line of lines) {
    const headerMatch = line.match(/^#{1,4}\s*(\d{4}-\d{2}-\d{2})(?::\s*|\s+-\s*)(.*)$/);
    if (headerMatch) {
      if (currentDate && (currentDecision || currentWhy)) {
        decisions.push({
          date: currentDate,
          decision: currentDecision.trim(),
          why: currentWhy.trim(),
        });
      }
      currentDate = headerMatch[1];
      currentDecision = headerMatch[2].trim();
      currentWhy = "";
      continue;
    }

    const trimmed = line.trim();
    const decMatch = trimmed.match(/^[-*]\s*(?:\*\*Decision\*\*|Decision):\s*(.*)$/i);
    if (decMatch) {
      currentDecision = decMatch[1].trim();
      continue;
    }

    const whyMatch = trimmed.match(/^[-*]\s*(?:\*\*Why\*\*|Why):\s*(.*)$/i);
    if (whyMatch) {
      currentWhy = whyMatch[1].trim();
      continue;
    }
  }

  if (currentDate && (currentDecision || currentWhy)) {
    decisions.push({
      date: currentDate,
      decision: currentDecision.trim(),
      why: currentWhy.trim(),
    });
  }

  return decisions;
}

export function appendDecision(
  existingContent: string,
  entry: DecisionItem
): string {
  const block = [
    `### ${entry.date}: ${entry.decision}`,
    `- **Decision**: ${entry.decision}`,
    `- **Why**: ${entry.why}`,
    "",
  ].join("\n");

  if (!existingContent.trim()) {
    return `# Architecture & Design Decisions\n\n${block}`;
  }

  return existingContent.trimEnd() + "\n\n" + block;
}
