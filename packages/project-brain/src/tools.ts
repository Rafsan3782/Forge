import type { ToolDescriptor } from "@forge/protocol";
import type { TaskItem } from "./types.ts";
import { ProjectBrain } from "./brain.ts";


export interface ToolResult {
  content: unknown;
  is_error: boolean;
}

export interface ToolProviderContract {
  listTools(): Promise<ToolDescriptor[]>;
  executeTool(
    name: string,
    args: Record<string, unknown>,
    signal?: AbortSignal
  ): Promise<ToolResult>;
}

export class BrainToolProvider implements ToolProviderContract {
  private brain: ProjectBrain;

  constructor(brainOrProjectRoot: ProjectBrain | string) {
    if (typeof brainOrProjectRoot === "string") {
      this.brain = new ProjectBrain(brainOrProjectRoot);
    } else {
      this.brain = brainOrProjectRoot;
    }
  }

  public getBrain(): ProjectBrain {
    return this.brain;
  }

  public async listTools(): Promise<ToolDescriptor[]> {
    return [
      {
        name: "brain.read",
        description:
          "Reads the project brain as a compact context pack within a token budget (GAME.md summary, active tasks, recent decisions, map index, memory notes).",
        input_schema: {
          type: "object",
          properties: {
            token_budget: {
              type: "number",
              description: "Maximum approximate tokens to include in context pack (default: 4000)",
            },
          },
        },
        category: "read",
        risk_level: "safe",
      },
      {
        name: "brain.update_section",
        description:
          "Patches a specific section by its stable ID in a project document (e.g. GAME.md or AGENTS.md) without overwriting the whole file. Performs 3-way merge if user edited the file.",
        input_schema: {
          type: "object",
          required: ["filename", "section_id", "content"],
          properties: {
            filename: {
              type: "string",
              description: "Target file path relative to project root (e.g. 'GAME.md', 'AGENTS.md')",
            },
            section_id: {
              type: "string",
              description: "Stable ID or heading slug of the section to patch (e.g. 'core-loop', 'controls')",
            },
            content: {
              type: "string",
              description: "New content for the section",
            },
            base_content: {
              type: "string",
              description: "Optional base content snapshot when agent turn started, for 3-way conflict detection",
            },
          },
        },
        category: "write",
        risk_level: "medium",
      },
      {
        name: "brain.add_task",
        description: "Adds a new task to TASKS.md with id, status, and acceptance checks.",
        input_schema: {
          type: "object",
          required: ["title"],
          properties: {
            id: {
              type: "string",
              description: "Optional custom task id (e.g. 'TASK-101')",
            },
            title: {
              type: "string",
              description: "Title of the task",
            },
            status: {
              type: "string",
              enum: ["todo", "in_progress", "done", "blocked"],
              description: "Initial status of the task (default: 'todo')",
            },
            acceptance_checks: {
              type: "array",
              items: { type: "string" },
              description: "List of verification criteria",
            },
            notes: {
              type: "string",
              description: "Optional task notes",
            },
          },
        },
        category: "write",
        risk_level: "low",
      },
      {
        name: "brain.update_task",
        description: "Updates an existing task in TASKS.md by task ID.",
        input_schema: {
          type: "object",
          required: ["task_id"],
          properties: {
            task_id: {
              type: "string",
              description: "ID of task to update (e.g. 'TASK-001')",
            },
            status: {
              type: "string",
              enum: ["todo", "in_progress", "done", "blocked"],
              description: "New status",
            },
            title: {
              type: "string",
              description: "Updated title",
            },
            acceptance_checks: {
              type: "array",
              items: { type: "string" },
              description: "Updated acceptance checks",
            },
            notes: {
              type: "string",
              description: "Updated notes",
            },
          },
        },
        category: "write",
        risk_level: "low",
      },
      {
        name: "brain.add_decision",
        description: "Appends a dated architecture or gameplay decision to DECISIONS.md (what and why).",
        input_schema: {
          type: "object",
          required: ["decision", "why"],
          properties: {
            decision: {
              type: "string",
              description: "Summary of the choice made",
            },
            why: {
              type: "string",
              description: "Rationale and alternatives considered",
            },
            date: {
              type: "string",
              description: "Optional ISO date (YYYY-MM-DD), defaults to today",
            },
          },
        },
        category: "write",
        risk_level: "low",
      },
      {
        name: "brain.remember",
        description:
          "Appends a note to .forge/memory.md. Automatically compacts older notes if the file exceeds size limit.",
        input_schema: {
          type: "object",
          required: ["note"],
          properties: {
            note: {
              type: "string",
              description: "Observation or knowledge to persist",
            },
          },
        },
        category: "write",
        risk_level: "low",
      },
    ];
  }

  public async executeTool(
    name: string,
    args: Record<string, unknown>
  ): Promise<ToolResult> {
    try {
      switch (name) {
        case "brain.read": {
          const budget = typeof args.token_budget === "number" ? args.token_budget : undefined;
          const pack = await this.brain.readContextPack({ tokenBudget: budget });
          return { content: pack, is_error: false };
        }

        case "brain.update_section": {
          const filename = String(args.filename || "GAME.md");
          const sectionId = String(args.section_id);
          const content = String(args.content);
          const baseContent = args.base_content ? String(args.base_content) : undefined;
          await this.brain.updateSection(filename, sectionId, content, baseContent);
          return { content: { success: true, filename, section_id: sectionId }, is_error: false };
        }

        case "brain.add_task": {
          const title = String(args.title);
          const status = (args.status as TaskItem["status"]) || "todo";
          const checks = Array.isArray(args.acceptance_checks)
            ? args.acceptance_checks.map(String)
            : [];
          const id = args.id ? String(args.id) : undefined;
          const notes = args.notes ? String(args.notes) : undefined;

          const created = await this.brain.addTask({
            id,
            title,
            status,
            acceptance_checks: checks,
            notes,
          });
          return { content: created, is_error: false };
        }

        case "brain.update_task": {
          const taskId = String(args.task_id);
          const updates: Partial<Omit<TaskItem, "id">> = {};
          if (args.status !== undefined) updates.status = args.status as TaskItem["status"];
          if (args.title !== undefined) updates.title = String(args.title);
          if (Array.isArray(args.acceptance_checks)) {
            updates.acceptance_checks = args.acceptance_checks.map(String);
          }
          if (args.notes !== undefined) updates.notes = String(args.notes);

          const updated = await this.brain.updateTask(taskId, updates);
          return { content: updated, is_error: false };
        }

        case "brain.add_decision": {
          const decision = String(args.decision);
          const why = String(args.why);
          const date = args.date ? String(args.date) : undefined;
          const entry = await this.brain.addDecision(decision, why, date);
          return { content: entry, is_error: false };
        }

        case "brain.remember": {
          const note = String(args.note);
          await this.brain.remember(note);
          return { content: { success: true, remembered: note }, is_error: false };
        }

        default:
          return {
            content: `Unknown brain tool: '${name}'`,
            is_error: true,
          };
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      return {
        content: msg,
        is_error: true,
      };
    }

  }
}
