import type { ContextPack, ContextPackOptions, TaskItem, DecisionItem } from "./types.ts";
import { parseSections } from "./markdown.ts";
import { parseTasks, parseDecisions } from "./tasks-decisions.ts";

export function estimateTokens(text: string): number {
  if (!text) return 0;
  return Math.ceil(text.length / 4);
}

export function buildContextPack(
  gameMd: string,
  tasksMd: string,
  decisionsMd: string,
  mapJson: Record<string, unknown> | null,
  memoryMd: string = "",
  options: ContextPackOptions = {}
): ContextPack {
  const budget = options.tokenBudget ?? 4000;
  let remainingTokens = budget;

  const gameSections = parseSections(gameMd);
  const coreSections = gameSections.filter((s) => {
    const id = s.id.toLowerCase();
    return (
      id.includes("concept") ||
      id.includes("pillar") ||
      id.includes("core-loop") ||
      id.includes("controls")
    );
  });

  let gameSummaryText = "";
  if (coreSections.length > 0) {
    gameSummaryText = coreSections
      .map((s) => `### ${s.heading}\n${s.content}`)
      .join("\n\n");
  } else {
    gameSummaryText = gameMd.slice(0, 800);
  }

  let gameTokens = estimateTokens(gameSummaryText);
  const maxGameTokens = Math.floor(budget * 0.35);
  if (gameTokens > maxGameTokens) {
    gameSummaryText = gameSummaryText.slice(0, maxGameTokens * 4) + "... [truncated]";
    gameTokens = estimateTokens(gameSummaryText);
  }
  remainingTokens -= gameTokens;

  const allTasks = parseTasks(tasksMd);
  const activeTasks = allTasks.filter(
    (t) => t.status === "in_progress" || t.status === "todo" || t.status === "blocked"
  );
  const selectedTasks: TaskItem[] = [];

  for (const task of activeTasks) {
    const taskTokens = estimateTokens(JSON.stringify(task));
    if (remainingTokens - taskTokens < 500) {
      break;
    }
    selectedTasks.push(task);
    remainingTokens -= taskTokens;
  }

  const allDecisions = parseDecisions(decisionsMd);
  const recentDecisions: DecisionItem[] = [];
  for (let i = allDecisions.length - 1; i >= 0; i--) {
    const dec = allDecisions[i];
    const decTokens = estimateTokens(JSON.stringify(dec));
    if (remainingTokens - decTokens < 300) {
      break;
    }
    recentDecisions.unshift(dec);
    remainingTokens -= decTokens;
  }

  let mapOverview: Record<string, unknown> | null = null;
  if (mapJson) {
    const scenes = Array.isArray(mapJson.scenes) ? mapJson.scenes : [];
    const entities = mapJson.entities && typeof mapJson.entities === "object" ? mapJson.entities : {};
    const scripts = Array.isArray(mapJson.scripts) ? mapJson.scripts : [];
    const assets = Array.isArray(mapJson.assets) ? mapJson.assets : [];
    const inputActions = Array.isArray(mapJson.input_actions) ? mapJson.input_actions : [];

    mapOverview = {
      scenes,
      entity_count: Object.keys(entities).length,
      sample_entities: Object.keys(entities).slice(0, 10),
      scripts: scripts.map((s: any) => ({
        path: s.path,
        lifecycle: s.lifecycle_functions,
      })),
      asset_count: assets.length,
      sample_assets: assets.slice(0, 10),
      input_actions: inputActions,
    };
  }

  const mapTokens = mapOverview ? estimateTokens(JSON.stringify(mapOverview)) : 0;
  remainingTokens -= mapTokens;

  const memoryNotes: string[] = [];
  if (memoryMd && remainingTokens > 100) {
    const lines = memoryMd
      .split(/\r?\n/)
      .map((l) => l.trim())
      .filter((l) => l.startsWith("- ["));
    for (let i = lines.length - 1; i >= 0; i--) {
      const line = lines[i];
      const tok = estimateTokens(line);
      if (remainingTokens - tok < 50) break;
      memoryNotes.unshift(line);
      remainingTokens -= tok;
    }
  }

  const totalEstimatedTokens = budget - remainingTokens;

  return {
    gameSummary: gameSummaryText,
    currentTasks: selectedTasks,
    recentDecisions,
    mapOverview,
    memoryNotes: memoryNotes.length > 0 ? memoryNotes : undefined,
    totalEstimatedTokens,
  };
}
