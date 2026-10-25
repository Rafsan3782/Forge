import type {
  EnginePingResult,
  SceneGetTreeResult,
  SceneCreateEntityResult,
  SceneSetComponentResult,
  SceneDeleteEntityResult,
  GameScreenshotParams,
  GameScreenshotResult,
  ProjectStopResult,
} from "@forge/protocol";

export type BridgeConnectionState = "disconnected" | "connecting" | "connected" | "error";

export interface BridgeClientOptions {
  host?: string;
  port: number;
  token: string;
  autoReconnect?: boolean;
  reconnectInterval?: number;
}

export class BridgeClient {
  private ws: WebSocket | null = null;
  private state: BridgeConnectionState = "disconnected";
  private stateListeners: Set<(state: BridgeConnectionState) => void> = new Set();
  private pendingRequests: Map<
    string | number,
    {
      resolve: (value: unknown) => void;
      reject: (reason: unknown) => void;
      timer: ReturnType<typeof setTimeout>;
    }
  > = new Map();
  private nextId = 1;
  private options: BridgeClientOptions;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private isExplicitDisconnect = false;

  constructor(options: BridgeClientOptions) {
    this.options = {
      host: "127.0.0.1",
      autoReconnect: false,
      reconnectInterval: 2000,
      ...options,
    };
  }

  public getState(): BridgeConnectionState {
    return this.state;
  }

  public isConnected(): boolean {
    return this.state === "connected" && this.ws?.readyState === WebSocket.OPEN;
  }

  public onStateChange(listener: (state: BridgeConnectionState) => void): () => void {
    this.stateListeners.add(listener);
    listener(this.state);
    return () => {
      this.stateListeners.delete(listener);
    };
  }

  private setState(newState: BridgeConnectionState) {
    if (this.state !== newState) {
      this.state = newState;
      for (const listener of this.stateListeners) {
        try {
          listener(newState);
        } catch (e) {
          console.error("Error in bridge state change listener:", e);
        }
      }
    }
  }

  public connect(): Promise<void> {
    this.isExplicitDisconnect = false;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (
      this.ws &&
      (this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING)
    ) {
      if (this.state === "connected") return Promise.resolve();
    }

    return new Promise((resolve, reject) => {
      const host = this.options.host || "127.0.0.1";
      const url = `ws://${host}:${this.options.port}/?token=${encodeURIComponent(this.options.token)}`;

      this.setState("connecting");

      try {
        const socket = new WebSocket(url);
        this.ws = socket;

        let resolved = false;

        socket.onopen = () => {
          resolved = true;
          this.setState("connected");
          resolve();
        };

        socket.onmessage = (event) => {
          this.handleMessage(event.data);
        };

        socket.onerror = (_err) => {
          if (!resolved) {
            resolved = true;
            this.setState("error");
            reject(new Error(`WebSocket connection failed to ${host}:${this.options.port}`));
          }
        };

        socket.onclose = () => {
          this.cleanupPendingRequests("WebSocket connection closed");
          if (this.state !== "error") {
            this.setState("disconnected");
          }
          if (!this.isExplicitDisconnect && this.options.autoReconnect) {
            this.scheduleReconnect();
          }
        };
      } catch (err) {
        this.setState("error");
        reject(err);
      }
    });
  }

  public disconnect(): void {
    this.isExplicitDisconnect = true;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.cleanupPendingRequests("Disconnected by user");
    this.setState("disconnected");
  }

  private scheduleReconnect() {
    if (this.reconnectTimer) return;
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      if (!this.isExplicitDisconnect && this.state !== "connected") {
        this.connect().catch(() => {
          // Handled via state transitions
        });
      }
    }, this.options.reconnectInterval || 2000);
  }

  private cleanupPendingRequests(reason: string) {
    for (const [id, req] of this.pendingRequests.entries()) {
      clearTimeout(req.timer);
      req.reject(new Error(`Request ${id} failed: ${reason}`));
    }
    this.pendingRequests.clear();
  }

  private handleMessage(data: unknown) {
    try {
      const rawText =
        typeof data === "string" ? data : new TextDecoder().decode(data as ArrayBuffer);
      const msg = JSON.parse(rawText);

      if ("id" in msg && msg.id !== null && msg.id !== undefined) {
        const pending = this.pendingRequests.get(msg.id);
        if (pending) {
          clearTimeout(pending.timer);
          this.pendingRequests.delete(msg.id);

          if ("error" in msg && msg.error) {
            pending.reject(
              new Error(
                msg.error.message || `RPC Error (${msg.error.code}): ${JSON.stringify(msg.error)}`,
              ),
            );
          } else {
            pending.resolve(msg.result);
          }
        }
      }
    } catch (e) {
      console.warn("Failed to parse incoming bridge WebSocket message:", e);
    }
  }

  public call<TResult = unknown>(
    method: string,
    params: Record<string, unknown> = {},
  ): Promise<TResult> {
    if (!this.isConnected() || !this.ws) {
      return Promise.reject(new Error("BridgeClient is not connected to engine runtime"));
    }

    const id = this.nextId++;
    const payload = JSON.stringify({
      jsonrpc: "2.0",
      id,
      method,
      params,
    });

    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        if (this.pendingRequests.has(id)) {
          this.pendingRequests.delete(id);
          reject(new Error(`RPC timeout calling method '${method}' (id ${id})`));
        }
      }, 10000);

      this.pendingRequests.set(id, {
        resolve: resolve as (val: unknown) => void,
        reject,
        timer,
      });

      try {
        this.ws!.send(payload);
      } catch (err) {
        clearTimeout(timer);
        this.pendingRequests.delete(id);
        reject(err);
      }
    });
  }

  // Typed engine methods
  public ping(timestamp: number = Date.now()): Promise<EnginePingResult> {
    return this.call<EnginePingResult>("engine.ping", { timestamp });
  }

  public getSceneTree(rootEntityId: string | null = null): Promise<SceneGetTreeResult> {
    return this.call<SceneGetTreeResult>("scene.get_tree", {
      root_entity_id: rootEntityId,
    });
  }

  public createEntity(
    name: string,
    parentId: string | null = null,
  ): Promise<SceneCreateEntityResult> {
    return this.call<SceneCreateEntityResult>("scene.create_entity", {
      name,
      parent_id: parentId,
    });
  }

  public setComponent(
    entityId: string,
    componentType: string,
    data: Record<string, unknown>,
  ): Promise<SceneSetComponentResult> {
    return this.call<SceneSetComponentResult>("scene.set_component", {
      entity_id: entityId,
      component_type: componentType,
      data,
    });
  }

  public deleteEntity(entityId: string): Promise<SceneDeleteEntityResult> {
    return this.call<SceneDeleteEntityResult>("scene.delete_entity", {
      entity_id: entityId,
    });
  }

  public screenshot(params: Partial<GameScreenshotParams> = {}): Promise<GameScreenshotResult> {
    const fullParams: GameScreenshotParams = {
      format: "png",
      ...params,
    };
    return this.call<GameScreenshotResult>(
      "game.screenshot",
      fullParams as Record<string, unknown>,
    );
  }

  public stop(): Promise<ProjectStopResult> {
    return this.call<ProjectStopResult>("project.stop", {});
  }

  public undo(): Promise<import("@forge/protocol").EditUndoResult> {
    return this.call("edit.undo", {});
  }

  public redo(): Promise<import("@forge/protocol").EditRedoResult> {
    return this.call("edit.redo", {});
  }

  public getHistory(): Promise<import("@forge/protocol").EditGetHistoryResult> {
    return this.call("edit.get_history", {});
  }

  public jumpHistory(targetId: string): Promise<import("@forge/protocol").EditJumpHistoryResult> {
    return this.call("edit.jump_history", { target_id: targetId });
  }

  public getComponentSchemas(): Promise<import("@forge/protocol").SceneGetComponentSchemasResult> {
    return this.call("scene.get_component_schemas", {});
  }

  public saveScene(path?: string): Promise<import("@forge/protocol").SceneSaveResult> {
    return this.call("scene.save", path ? { path } : {});
  }

  public getDirty(): Promise<import("@forge/protocol").SceneGetDirtyResult> {
    return this.call("scene.get_dirty", {});
  }

  public reparentEntity(
    entityId: string,
    newParentId: string | null = null,
  ): Promise<import("@forge/protocol").SceneReparentEntityResult> {
    return this.call("scene.reparent_entity", {
      entity_id: entityId,
      new_parent_id: newParentId,
    });
  }

  public duplicateEntity(
    entityId: string,
    newEntityId?: string,
  ): Promise<import("@forge/protocol").SceneDuplicateEntityResult> {
    return this.call("scene.duplicate_entity", {
      entity_id: entityId,
      new_entity_id: newEntityId,
    });
  }

  public removeComponent(
    entityId: string,
    componentName: string,
  ): Promise<import("@forge/protocol").SceneRemoveComponentResult> {
    return this.call("scene.remove_component", {
      entity_id: entityId,
      component_name: componentName,
    });
  }

  public playStart(): Promise<import("@forge/protocol").PlayStartResult> {
    return this.call("play.start", {});
  }

  public playPause(): Promise<import("@forge/protocol").PlayPauseResult> {
    return this.call("play.pause", {});
  }

  public playStep(frames = 1): Promise<import("@forge/protocol").PlayStepResult> {
    return this.call("play.step", { frames });
  }

  public playStop(): Promise<import("@forge/protocol").PlayStopResult> {
    return this.call("play.stop", {});
  }

  public simulateInput(
    events: import("@forge/protocol").SimulateInputEvent[],
  ): Promise<import("@forge/protocol").GameSimulateInputResult> {
    return this.call("game.simulate_input", { events });
  }
}
