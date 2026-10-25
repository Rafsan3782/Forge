import { create } from "zustand";
import { useRuntimeStore } from "./runtimeStore";

export type PlayState = "edit" | "playing" | "paused";

export type GizmoTool = "translate" | "rotate" | "scale";

export interface PlayStoreState {
  playState: PlayState;
  activeGizmo: GizmoTool;
  gridSnapEnabled: boolean;
  angleSnapEnabled: boolean;
  showGrid: boolean;

  // Actions
  play: () => Promise<void>;
  pause: () => Promise<void>;
  step: () => Promise<void>;
  stop: () => Promise<void>;
  setActiveGizmo: (tool: GizmoTool) => void;
  toggleGridSnap: () => void;
  toggleAngleSnap: () => void;
  toggleShowGrid: () => void;
}

export const usePlayStore = create<PlayStoreState>((set, get) => ({
  playState: "edit",
  activeGizmo: "translate",
  gridSnapEnabled: true,
  angleSnapEnabled: true,
  showGrid: true,

  play: async () => {
    const { bridgeClient } = useRuntimeStore.getState();
    if (bridgeClient && bridgeClient.isConnected()) {
      try {
        await bridgeClient.playStart();
      } catch (err) {
        console.error("Failed to start play:", err);
      }
    }
    set({ playState: "playing" });
  },

  pause: async () => {
    const { bridgeClient } = useRuntimeStore.getState();
    if (bridgeClient && bridgeClient.isConnected()) {
      try {
        await bridgeClient.playPause();
      } catch (err) {
        console.error("Failed to pause play:", err);
      }
    }
    set({ playState: "paused" });
  },

  step: async () => {
    const { bridgeClient } = useRuntimeStore.getState();
    if (bridgeClient && bridgeClient.isConnected()) {
      try {
        await bridgeClient.playStep();
      } catch (err) {
        console.error("Failed to step play:", err);
      }
    }
    set({ playState: "paused" });
  },

  stop: async () => {
    const { bridgeClient } = useRuntimeStore.getState();
    if (bridgeClient && bridgeClient.isConnected()) {
      try {
        await bridgeClient.playStop();
      } catch (err) {
        console.error("Failed to stop play:", err);
      }
    }
    set({ playState: "edit" });
  },

  setActiveGizmo: (tool: GizmoTool) => {
    set({ activeGizmo: tool });
  },

  toggleGridSnap: () => {
    set((state) => ({ gridSnapEnabled: !state.gridSnapEnabled }));
  },

  toggleAngleSnap: () => {
    set((state) => ({ angleSnapEnabled: !state.angleSnapEnabled }));
  },

  toggleShowGrid: () => {
    set((state) => ({ showGrid: !state.showGrid }));
  },
}));
