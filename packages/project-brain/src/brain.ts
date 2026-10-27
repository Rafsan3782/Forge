import * as fs from "node:fs/promises";
import * as path from "node:path";
import type {
  ContextPack,
  ContextPackOptions,
  TaskItem,
  DecisionItem,
  ProjectBrainContract,
} from "./types.ts";
import { parseSections, patchSection, mergeSectionsThreeWay } from "./markdown.ts";
import { parseTasks, serializeTasks, parseDecisions, appendDecision } from "./tasks-decisions.ts";
import { appendMemoryNote } from "./memory.ts";
import { buildContextPack } from "./context-pack.ts";




export class ProjectBrain implements ProjectBrainContract {
  private projectRoot: string;
  private inMemoryKnowledge: Map<string, string> = new Map();

  constructor(projectRoot: string) {
    this.projectRoot = path.resolve(projectRoot);
  }

  public getProjectRoot(): string {
    return this.projectRoot;
  }

  private getFilePath(filename: string): string {
    return path.join(this.projectRoot, filename);
  }

  private async readFileSafe(relPath: string): Promise<string> {
    try {
      return await fs.readFile(this.getFilePath(relPath), "utf8");
    } catch {
      return "";
    }
  }

  private async writeFileSafe(relPath: string, content: string): Promise<void> {
    const fullPath = this.getFilePath(relPath);
    await fs.mkdir(path.dirname(fullPath), { recursive: true });
    await fs.writeFile(fullPath, content, "utf8");
  }

  public async recordKnowledge(topic: string, content: string): Promise<void> {
    this.inMemoryKnowledge.set(topic, content);
    await this.remember(`[${topic}]: ${content}`);
  }

  public async queryKnowledge(query: string): Promise<string[]> {
    const results: string[] = [];
    const q = query.toLowerCase();

    for (const [topic, content] of this.inMemoryKnowledge.entries()) {
      if (topic.toLowerCase().includes(q) || content.toLowerCase().includes(q)) {
        results.push(`[${topic}]: ${content}`);
      }
    }

    const gameMd = await this.readFileSafe("GAME.md");
    if (gameMd) {
      const sections = parseSections(gameMd);
      for (const s of sections) {
        if (s.heading.toLowerCase().includes(q) || s.content.toLowerCase().includes(q)) {
          results.push(`[GAME.md > ${s.heading}]: ${s.content.slice(0, 150)}`);
        }
      }
    }

    const tasksMd = await this.readFileSafe("TASKS.md");
    if (tasksMd) {
      const tasks = parseTasks(tasksMd);
      for (const t of tasks) {
        if (t.title.toLowerCase().includes(q) || t.id.toLowerCase().includes(q)) {
          results.push(`[TASKS.md > ${t.id}]: (${t.status}) ${t.title}`);
        }
      }
    }

    const decisionsMd = await this.readFileSafe("DECISIONS.md");
    if (decisionsMd) {
      const decisions = parseDecisions(decisionsMd);
      for (const d of decisions) {
        if (d.decision.toLowerCase().includes(q) || d.why.toLowerCase().includes(q)) {
          results.push(`[DECISIONS.md > ${d.date}]: ${d.decision} - ${d.why}`);
        }
      }
    }

    return results;
  }

  public async getContextSnapshot(): Promise<Record<string, unknown>> {
    const pack = await this.readContextPack();
    return {
      project_root: this.projectRoot,
      game_summary: pack.gameSummary,
      current_tasks: pack.currentTasks,
      recent_decisions: pack.recentDecisions,
      map_overview: pack.mapOverview,
      memory_notes: pack.memoryNotes,
      in_memory_records: Object.fromEntries(this.inMemoryKnowledge.entries()),
    };
  }

  public async readContextPack(options?: ContextPackOptions): Promise<ContextPack> {
    const gameMd = await this.readFileSafe("GAME.md");
    const tasksMd = await this.readFileSafe("TASKS.md");
    const decisionsMd = await this.readFileSafe("DECISIONS.md");
    const memoryMd = await this.readFileSafe(".forge/memory.md");

    let mapJson: Record<string, unknown> | null = null;
    const mapStr = await this.readFileSafe(".forge/map.json");
    if (mapStr) {
      try {
        mapJson = JSON.parse(mapStr);
      } catch {
        mapJson = null;
      }
    }

    return buildContextPack(gameMd, tasksMd, decisionsMd, mapJson, memoryMd, options);
  }

  public async updateSection(
    filename: string,
    sectionId: string,
    newContent: string,
    baseContentForMerge?: string
  ): Promise<void> {
    const currentOnDisk = await this.readFileSafe(filename);

    if (baseContentForMerge && baseContentForMerge !== currentOnDisk) {
      const agentProposed = patchSection(baseContentForMerge, sectionId, newContent);
      const merge = mergeSectionsThreeWay(baseContentForMerge, currentOnDisk, agentProposed);
      await this.writeFileSafe(filename, merge.mergedContent);
      if (merge.conflicts.length > 0) {
        throw new Error(
          `Concurrent edit conflict in ${filename} at section '${merge.conflicts[0].sectionId}'. Manual resolution required.`
        );
      }
      return;
    }

    const patched = patchSection(currentOnDisk, sectionId, newContent);
    await this.writeFileSafe(filename, patched);
  }

  public async addTask(task: Omit<TaskItem, "id"> & { id?: string }): Promise<TaskItem> {
    const tasksMd = await this.readFileSafe("TASKS.md");
    const tasks = parseTasks(tasksMd);

    const nextId = task.id || `TASK-${String(tasks.length + 1).padStart(3, "0")}`;
    const newTask: TaskItem = {
      id: nextId,
      title: task.title,
      status: task.status || "todo",
      acceptance_checks: task.acceptance_checks || [],
      notes: task.notes,
    };

    tasks.push(newTask);
    await this.writeFileSafe("TASKS.md", serializeTasks(tasks));
    return newTask;
  }

  public async updateTask(
    taskId: string,
    updates: Partial<Omit<TaskItem, "id">>
  ): Promise<TaskItem> {
    const tasksMd = await this.readFileSafe("TASKS.md");
    const tasks = parseTasks(tasksMd);

    const task = tasks.find((t) => t.id.toLowerCase() === taskId.toLowerCase());
    if (!task) {
      throw new Error(`Task '${taskId}' not found in TASKS.md`);
    }

    if (updates.status !== undefined) task.status = updates.status;
    if (updates.title !== undefined) task.title = updates.title;
    if (updates.acceptance_checks !== undefined) task.acceptance_checks = updates.acceptance_checks;
    if (updates.notes !== undefined) task.notes = updates.notes;

    await this.writeFileSafe("TASKS.md", serializeTasks(tasks));
    return task;
  }

  public async addDecision(decision: string, why: string, date?: string): Promise<DecisionItem> {
    const decisionsMd = await this.readFileSafe("DECISIONS.md");
    const entryDate = date || new Date().toISOString().slice(0, 10);
    const item: DecisionItem = {
      date: entryDate,
      decision,
      why,
    };

    const updated = appendDecision(decisionsMd, item);
    await this.writeFileSafe("DECISIONS.md", updated);
    return item;
  }

  public async remember(note: string, maxBytes?: number): Promise<void> {
    const memoryMd = await this.readFileSafe(".forge/memory.md");
    const updated = appendMemoryNote(memoryMd, note, maxBytes);
    await this.writeFileSafe(".forge/memory.md", updated);
  }
}
