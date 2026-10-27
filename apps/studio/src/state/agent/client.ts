import type { AgentCommand, AgentEvent } from "@forge/protocol";

export type AgentEventCallback = (event: AgentEvent) => void;

export interface AgentClientOptions {
  fixtureMode?: boolean;
  projectPath?: string;
  bridgePort?: number;
  bridgeToken?: string;
}

const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/**
 * Typed client communicating with the agent sidecar via Tauri IPC or replaying JSONL fixtures.
 */
export class AgentClient {
  private listeners = new Set<AgentEventCallback>();
  private stderrListeners = new Set<(line: string) => void>();
  private statusListeners = new Set<(status: string) => void>();
  private unlistenIpc: (() => void) | null = null;
  private fixtureInterval: ReturnType<typeof setInterval> | null = null;
  private isFixture = false;

  constructor(private options: AgentClientOptions = {}) {
    this.isFixture = !!options.fixtureMode;
  }

  public isFixtureMode(): boolean {
    return this.isFixture;
  }

  public onEvent(callback: AgentEventCallback): () => void {
    this.listeners.add(callback);
    return () => this.listeners.delete(callback);
  }

  public onStderr(callback: (line: string) => void): () => void {
    this.stderrListeners.add(callback);
    return () => this.stderrListeners.delete(callback);
  }

  public onStatus(callback: (status: string) => void): () => void {
    this.statusListeners.add(callback);
    return () => this.statusListeners.delete(callback);
  }

  public async connect(): Promise<void> {
    if (this.isFixture) {
      this.notifyStatus("running");
      return;
    }

    if (!isTauri) {
      // In browser web preview without Tauri, default to simulated status
      this.notifyStatus("running");
      return;
    }

    try {
      const { listen } = await import("@tauri-apps/api/event");
      const unlistenEvent = await listen<AgentEvent>("forge://agent-event", (e) => {
        this.notifyEvent(e.payload);
      });

      const unlistenStderr = await listen<string>("forge://agent-stderr", (e) => {
        this.notifyStderr(e.payload);
      });

      const unlistenStatus = await listen<string>("forge://agent-status", (e) => {
        this.notifyStatus(e.payload);
      });

      this.unlistenIpc = () => {
        unlistenEvent();
        unlistenStderr();
        unlistenStatus();
      };
    } catch (err) {
      console.warn("Failed to subscribe to Tauri agent IPC events:", err);
    }
  }

  public disconnect(): void {
    if (this.fixtureInterval) {
      clearInterval(this.fixtureInterval);
      this.fixtureInterval = null;
    }
    if (this.unlistenIpc) {
      this.unlistenIpc();
      this.unlistenIpc = null;
    }
    this.notifyStatus("stopped");
  }

  public async sendCommand(cmd: AgentCommand): Promise<void> {
    if (this.isFixture) {
      // Fixture mode simulates response based on command
      return;
    }

    if (!isTauri) {
      console.info("[AgentClient mock] Sending command:", cmd);
      return;
    }

    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("send_agent_command", { command: cmd });
  }

  /**
   * Replays an array of AgentEvent records or a JSONL string sequentially.
   * Useful for fixture testing and offline development.
   */
  public replayEvents(events: AgentEvent[] | string, delayMs = 100): Promise<void> {
    const list: AgentEvent[] =
      typeof events === "string"
        ? events
            .split("\n")
            .map((line) => line.trim())
            .filter((line) => line.length > 0)
            .map((line) => JSON.parse(line) as AgentEvent)
        : events;

    return new Promise<void>((resolve) => {
      let index = 0;
      this.notifyStatus("running");

      this.fixtureInterval = setInterval(() => {
        if (index >= list.length) {
          if (this.fixtureInterval) {
            clearInterval(this.fixtureInterval);
            this.fixtureInterval = null;
          }
          resolve();
          return;
        }

        const ev = list[index];
        index++;
        this.notifyEvent(ev);
      }, delayMs);
    });
  }

  private notifyEvent(event: AgentEvent) {
    for (const listener of this.listeners) {
      try {
        listener(event);
      } catch (err) {
        console.error("Error in AgentEvent listener:", err);
      }
    }
  }

  private notifyStderr(line: string) {
    for (const listener of this.stderrListeners) {
      try {
        listener(line);
      } catch (err) {
        console.error("Error in Agent stderr listener:", err);
      }
    }
  }

  private notifyStatus(status: string) {
    for (const listener of this.statusListeners) {
      try {
        listener(status);
      } catch (err) {
        console.error("Error in Agent status listener:", err);
      }
    }
  }
}
