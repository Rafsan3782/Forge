import { create } from "zustand";
import type {
  AgentCommand,
  AgentEvent,
  AgentSettings,
  BudgetUsage,
  Checkpoint,
  PermissionRequest,
  Plan,
} from "@forge/protocol";
import { AgentClient } from "./client";

export type AgentStatus = "stopped" | "starting" | "running" | "crashed";

export interface ToolCallItem {
  id: string;
  name: string;
  arguments: Record<string, unknown>;
  result?: unknown;
  status: "running" | "completed" | "failed";
}

export interface AgentMessageItem {
  id: string;
  role: "user" | "agent" | "system";
  content: string;
  timestamp: number;
}

export interface AgentSessionState {
  runId: string | null;
  status: AgentStatus;
  crashedReason: string | null;
  messages: AgentMessageItem[];
  currentPlan: Plan | null;
  toolCalls: Record<string, ToolCallItem>;
  pendingPermissions: PermissionRequest[];
  checkpoints: Checkpoint[];
  budgetUsage: BudgetUsage | null;
  settings: AgentSettings | null;
  client: AgentClient | null;
  isFixtureMode: boolean;

  // Actions
  init: (options?: { fixtureMode?: boolean; projectPath?: string }) => Promise<void>;
  startRun: (prompt: string) => Promise<void>;
  sendMessage: (message: string) => Promise<void>;
  stopRun: () => Promise<void>;
  pauseRun: () => Promise<void>;
  resumeRun: () => Promise<void>;
  resolvePermission: (requestId: string, approved: boolean) => Promise<void>;
  revertCheckpoint: (checkpointId: string) => Promise<void>;
  loadSettings: (projectPath?: string) => Promise<void>;
  saveSettings: (settings: AgentSettings, projectPath?: string) => Promise<void>;
  loadApiKey: () => Promise<boolean>;
  saveApiKey: (key: string) => Promise<void>;
  deleteApiKey: () => Promise<void>;
  applyEvent: (event: AgentEvent) => void;
  replayFixture: (fixtureJsonl: string, delayMs?: number) => Promise<void>;
  setFixtureCrashed: (reason?: string) => void;
}

const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export const useAgentStore = create<AgentSessionState>((set, get) => ({
  runId: null,
  status: "stopped",
  crashedReason: null,
  messages: [],
  currentPlan: null,
  toolCalls: {},
  pendingPermissions: [],
  checkpoints: [],
  budgetUsage: null,
  settings: null,
  client: null,
  isFixtureMode: false,

  init: async (options = {}) => {
    const fixtureMode =
      options.fixtureMode ??
      (typeof window !== "undefined" &&
        (window.location.search.includes("fixture") ||
          window.location.search.includes("mode=fixture")));

    const client = new AgentClient({
      fixtureMode,
      projectPath: options.projectPath,
    });

    set({ client, isFixtureMode: fixtureMode });

    client.onEvent((event) => {
      get().applyEvent(event);
    });

    client.onStatus((status) => {
      set({
        status: status as AgentStatus,
        crashedReason: status === "crashed" ? get().crashedReason || "Process crashed" : null,
      });
    });

    if (isTauri && !fixtureMode) {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        await listen<{ exit_code: number | null; reason: string }>(
          "forge://agent-crashed",
          (e) => {
            set({ status: "crashed", crashedReason: e.payload.reason });
          },
        );

        // Also forward stderr lines to logStore
        await listen<string>("forge://agent-stderr", async (e) => {
          try {
            const { useLogStore } = await import("../../stores/logStore");
            useLogStore.getState().addLog({
              level: "error",
              source: "agent",
              message: e.payload,
            });
          } catch {
            // ignore if logStore is not loaded
          }
        });
      } catch (err) {
        console.warn("Could not register agent crash listener:", err);
      }
    }

    await client.connect();
    if (!fixtureMode) {
      await get().loadSettings(options.projectPath);
    }
  },

  applyEvent: (event: AgentEvent) => {
    const { type, run_id, timestamp, data = {} } = event;

    switch (type) {
      case "run_started": {
        const prompt = (data.prompt as string) || "";
        set((state) => ({
          runId: run_id,
          status: "running",
          crashedReason: null,
          messages: prompt
            ? [
                ...state.messages,
                {
                  id: `user-${timestamp}`,
                  role: "user",
                  content: prompt,
                  timestamp,
                },
              ]
            : state.messages,
        }));
        break;
      }

      case "message_delta": {
        const delta = (data.delta as string) || "";
        set((state) => {
          const msgs = [...state.messages];
          const lastMsg = msgs[msgs.length - 1];
          if (lastMsg && lastMsg.role === "agent") {
            msgs[msgs.length - 1] = {
              ...lastMsg,
              content: lastMsg.content + delta,
            };
          } else {
            msgs.push({
              id: `agent-${timestamp}`,
              role: "agent",
              content: delta,
              timestamp,
            });
          }
          return { messages: msgs };
        });
        break;
      }

      case "message_done": {
        const fullContent = (data.content as string) || "";
        set((state) => {
          const msgs = [...state.messages];
          const lastMsg = msgs[msgs.length - 1];
          if (lastMsg && lastMsg.role === "agent") {
            msgs[msgs.length - 1] = {
              ...lastMsg,
              content: fullContent || lastMsg.content,
            };
          } else if (fullContent) {
            msgs.push({
              id: `agent-${timestamp}`,
              role: "agent",
              content: fullContent,
              timestamp,
            });
          }
          return { messages: msgs };
        });
        break;
      }

      case "plan_updated": {
        if (data.plan) {
          set({ currentPlan: data.plan as Plan });
        }
        break;
      }

      case "tool_call_started": {
        const callId = (data.call_id as string) || `call-${timestamp}`;
        const name = (data.tool_name as string) || "";
        const args = (data.arguments as Record<string, unknown>) || {};
        set((state) => ({
          toolCalls: {
            ...state.toolCalls,
            [callId]: {
              id: callId,
              name,
              arguments: args,
              status: "running",
            },
          },
        }));
        break;
      }

      case "tool_call_finished": {
        const callId = (data.call_id as string) || "";
        const result = data.result;
        set((state) => {
          const existing = state.toolCalls[callId];
          if (!existing) return state;
          return {
            toolCalls: {
              ...state.toolCalls,
              [callId]: {
                ...existing,
                result,
                status: "completed",
              },
            },
          };
        });
        break;
      }

      case "permission_requested": {
        if (data.request) {
          const req = data.request as PermissionRequest;
          set((state) => ({
            pendingPermissions: [...state.pendingPermissions, req],
          }));
        }
        break;
      }

      case "permission_resolved": {
        const reqId = (data.request_id as string) || "";
        set((state) => ({
          pendingPermissions: state.pendingPermissions.filter((r) => r.id !== reqId),
        }));
        break;
      }

      case "checkpoint_created": {
        if (data.checkpoint) {
          const chk = data.checkpoint as Checkpoint;
          set((state) => ({
            checkpoints: [...state.checkpoints, chk],
          }));
        }
        break;
      }

      case "budget_update": {
        if (data.usage) {
          set({ budgetUsage: data.usage as BudgetUsage });
        }
        break;
      }

      case "run_paused": {
        set({ status: "stopped" });
        break;
      }

      case "run_finished": {
        set({ status: "stopped" });
        break;
      }

      case "error": {
        const errMsg = (data.error as string) || (data.message as string) || "Agent runtime error";
        set((state) => ({
          messages: [
            ...state.messages,
            {
              id: `err-${timestamp}`,
              role: "system",
              content: `Error: ${errMsg}`,
              timestamp,
            },
          ],
        }));
        break;
      }
    }
  },

  startRun: async (prompt: string) => {
    const { client } = get();
    const cmd: AgentCommand = {
      command: "start_run",
      payload: { prompt },
    };
    if (client) {
      await client.sendCommand(cmd);
    }
  },

  sendMessage: async (message: string) => {
    const { client, runId } = get();
    const cmd: AgentCommand = {
      command: "send_message",
      run_id: runId,
      payload: { message },
    };
    if (client) {
      await client.sendCommand(cmd);
    }
  },

  stopRun: async () => {
    const { client, runId } = get();
    const cmd: AgentCommand = {
      command: "stop",
      run_id: runId,
    };
    if (client) {
      await client.sendCommand(cmd);
    }
  },

  pauseRun: async () => {
    const { client, runId } = get();
    const cmd: AgentCommand = {
      command: "pause",
      run_id: runId,
    };
    if (client) {
      await client.sendCommand(cmd);
    }
  },

  resumeRun: async () => {
    const { client, runId } = get();
    const cmd: AgentCommand = {
      command: "resume",
      run_id: runId,
    };
    if (client) {
      await client.sendCommand(cmd);
    }
  },

  resolvePermission: async (requestId: string, approved: boolean) => {
    const { client, runId } = get();
    const cmd: AgentCommand = {
      command: "resolve_permission",
      run_id: runId,
      payload: { request_id: requestId, approved },
    };
    set((state) => ({
      pendingPermissions: state.pendingPermissions.filter((r) => r.id !== requestId),
    }));
    if (client) {
      await client.sendCommand(cmd);
    }
  },

  revertCheckpoint: async (checkpointId: string) => {
    const { client, runId } = get();
    const cmd: AgentCommand = {
      command: "revert_checkpoint",
      run_id: runId,
      payload: { checkpoint_id: checkpointId },
    };
    if (client) {
      await client.sendCommand(cmd);
    }
  },

  loadSettings: async (projectPath?: string) => {
    if (!isTauri) return;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const settings = await invoke<AgentSettings>("read_agent_settings", { projectPath });
      set({ settings });
    } catch (err) {
      console.warn("Failed to read agent settings:", err);
    }
  },

  saveSettings: async (settings: AgentSettings, projectPath?: string) => {
    set({ settings });
    if (!isTauri) return;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("write_agent_settings", { settings, projectPath });
    } catch (err) {
      console.warn("Failed to write agent settings:", err);
    }
  },

  loadApiKey: async () => {
    if (!isTauri) return false;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      return await invoke<boolean>("has_api_key");
    } catch {
      return false;
    }
  },

  saveApiKey: async (apiKey: string) => {
    if (!isTauri) return;
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("set_api_key", { apiKey });
  },

  deleteApiKey: async () => {
    if (!isTauri) return;
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("delete_api_key");
  },

  replayFixture: async (fixtureJsonl: string, delayMs = 50) => {
    const { client } = get();
    if (client) {
      await client.replayEvents(fixtureJsonl, delayMs);
    }
  },

  setFixtureCrashed: (reason = "Sidecar process crashed (SIGKILL / Access Violation)") => {
    set({ status: "crashed", crashedReason: reason });
  },
}));

// -----------------------------------------------------------------------------
// Typed Selectors
// -----------------------------------------------------------------------------
export const selectAgentStatus = (state: AgentSessionState) => state.status;
export const selectCrashedReason = (state: AgentSessionState) => state.crashedReason;
export const selectAgentMessages = (state: AgentSessionState) => state.messages;
export const selectCurrentPlan = (state: AgentSessionState) => state.currentPlan;
export const selectToolCalls = (state: AgentSessionState) => Object.values(state.toolCalls);
export const selectPendingPermissions = (state: AgentSessionState) => state.pendingPermissions;
export const selectCheckpoints = (state: AgentSessionState) => state.checkpoints;
export const selectBudgetUsage = (state: AgentSessionState) => state.budgetUsage;
export const selectAgentSettings = (state: AgentSessionState) => state.settings;
