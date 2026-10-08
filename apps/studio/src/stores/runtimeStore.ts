import { create } from "zustand";
import { BridgeClient, type BridgeConnectionState } from "@forge/agent";

export type RuntimeLifecycleStatus = "stopped" | "starting" | "running" | "crashed";
export type ViewportRenderMode = "embed" | "stream";

export interface ViewportBounds {
  x: number;
  y: number;
  width: number;
  height: number;
  visible: boolean;
}

export interface RuntimeStoreState {
  status: RuntimeLifecycleStatus;
  port: number | null;
  token: string | null;
  pid: number | null;
  scene: string;
  mode: ViewportRenderMode;
  bridgeState: BridgeConnectionState;
  bridgeClient: BridgeClient | null;
  streamFrame: string | null;
  fps: number;
  crashedReason: string | null;

  // Actions
  launchRuntime: (scene?: string, mode?: ViewportRenderMode) => Promise<void>;
  stopRuntime: () => Promise<void>;
  restartRuntime: () => Promise<void>;
  setMode: (mode: ViewportRenderMode) => Promise<void>;
  updateBounds: (bounds: ViewportBounds) => Promise<void>;
  focusViewport: () => Promise<void>;
  focusStudio: () => Promise<void>;
  initListeners: () => () => void;
  pollStreamFrame: () => Promise<void>;
}

const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export const useRuntimeStore = create<RuntimeStoreState>((set, get) => ({
  status: "stopped",
  port: null,
  token: null,
  pid: null,
  scene: "templates/hello/scene.json",
  mode: "embed",
  bridgeState: "disconnected",
  bridgeClient: null,
  streamFrame: null,
  fps: 60.0,
  crashedReason: null,

  launchRuntime: async (scene = "templates/hello/scene.json", mode) => {
    const currentMode = mode ?? get().mode;
    set({ status: "starting", scene, mode: currentMode, crashedReason: null });

    if (!isTauri) {
      // Mock for browser dev preview
      setTimeout(() => {
        set({ status: "running", port: 9876 });
      }, 500);
      return;
    }

    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("launch_runtime", { scene, mode: currentMode });
    } catch (err) {
      console.error("Failed to launch runtime:", err);
      set({ status: "crashed", crashedReason: String(err) });
    }
  },

  stopRuntime: async () => {
    const { bridgeClient } = get();
    if (bridgeClient) {
      bridgeClient.disconnect();
    }
    set({ status: "stopped", bridgeState: "disconnected", bridgeClient: null });

    if (isTauri) {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("stop_runtime");
      } catch (err) {
        console.warn("Failed to stop runtime:", err);
      }
    }
  },

  restartRuntime: async () => {
    const { scene, mode, launchRuntime, stopRuntime } = get();
    await stopRuntime();
    await launchRuntime(scene, mode);
  },

  setMode: async (mode: ViewportRenderMode) => {
    set({ mode });
    if (isTauri) {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("set_viewport_mode", { mode });
      } catch (err) {
        console.warn("Failed to set viewport mode:", err);
      }
    }
  },

  updateBounds: async (bounds: ViewportBounds) => {
    if (!isTauri) return;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("update_viewport_bounds", { bounds });
    } catch (err) {
      console.warn("Failed to update viewport bounds:", err);
    }
  },

  focusViewport: async () => {
    if (!isTauri) return;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("focus_viewport");
    } catch (err) {
      console.warn("Failed to focus viewport:", err);
    }
  },

  focusStudio: async () => {
    if (!isTauri) return;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("focus_studio");
    } catch (err) {
      console.warn("Failed to focus studio:", err);
    }
  },

  pollStreamFrame: async () => {
    const { bridgeClient, mode } = get();
    if (mode !== "stream" || !bridgeClient || !bridgeClient.isConnected()) return;

    try {
      const res = await bridgeClient.screenshot({ format: "png" });
      if (res.data_base64) {
        set({ streamFrame: res.data_base64 });
      }
    } catch {
      // Frame skip
    }
  },

  initListeners: () => {
    if (!isTauri) return () => {};

    let unlistenReady: (() => void) | undefined;
    let unlistenCrashed: (() => void) | undefined;
    let unlistenStatus: (() => void) | undefined;

    import("@tauri-apps/api/event").then(({ listen }) => {
      listen<{ port: number; token: string }>("forge://bridge-ready", async (event) => {
        const { port, token } = event.payload;
        set({ port, token, status: "running" });

        // Instantiate BridgeClient
        const client = new BridgeClient({
          port,
          token,
          autoReconnect: true,
        });

        client.onStateChange((bridgeState) => {
          set({ bridgeState });
        });

        try {
          await client.connect();
          set({ bridgeClient: client });
        } catch (err) {
          console.error("Failed to connect bridge client to engine:", err);
        }
      }).then((fn) => {
        unlistenReady = fn;
      });

      listen<{ exit_code: number | null; reason: string }>("forge://runtime-crashed", (event) => {
        const { bridgeClient } = get();
        if (bridgeClient) {
          bridgeClient.disconnect();
        }
        set({
          status: "crashed",
          bridgeState: "disconnected",
          bridgeClient: null,
          crashedReason: event.payload.reason,
        });
      }).then((fn) => {
        unlistenCrashed = fn;
      });

      listen<string>("forge://runtime-status", (event) => {
        const statusStr = event.payload;
        if (statusStr === "running" || statusStr === "stopped" || statusStr === "crashed") {
          set({ status: statusStr });
        }
      }).then((fn) => {
        unlistenStatus = fn;
      });
    });

    return () => {
      unlistenReady?.();
      unlistenCrashed?.();
      unlistenStatus?.();
    };
  },
}));
