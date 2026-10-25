import { create } from "zustand";
import type { HistoryEntry, ComponentSchema } from "@forge/protocol";
import { useRuntimeStore } from "./runtimeStore";
import { useSceneStore } from "./sceneStore";

export interface EditStoreState {
  undoStack: HistoryEntry[];
  redoStack: HistoryEntry[];
  canUndo: boolean;
  canRedo: boolean;
  isDirty: boolean;
  scenePath: string | null;
  componentSchemas: Record<string, ComponentSchema>;
  isLoadingSchemas: boolean;

  fetchHistory: () => Promise<void>;
  fetchDirty: () => Promise<void>;
  fetchComponentSchemas: () => Promise<void>;
  undo: () => Promise<void>;
  redo: () => Promise<void>;
  jumpHistory: (id: string) => Promise<void>;
  saveScene: () => Promise<void>;
  setDirty: (dirty: boolean) => void;
}

export const useEditStore = create<EditStoreState>((set, get) => ({
  undoStack: [],
  redoStack: [],
  canUndo: false,
  canRedo: false,
  isDirty: false,
  scenePath: null,
  componentSchemas: {},
  isLoadingSchemas: false,

  fetchHistory: async () => {
    const bridge = useRuntimeStore.getState().bridgeClient;
    if (!bridge || !bridge.isConnected()) return;

    try {
      const res = await bridge.getHistory();
      set({
        undoStack: res.undo_stack,
        redoStack: res.redo_stack,
        canUndo: res.can_undo,
        canRedo: res.can_redo,
      });
    } catch (err) {
      console.error("Failed to fetch history:", err);
    }
  },

  fetchDirty: async () => {
    const bridge = useRuntimeStore.getState().bridgeClient;
    if (!bridge || !bridge.isConnected()) return;

    try {
      const res = await bridge.getDirty();
      set({
        isDirty: res.is_dirty,
        scenePath: res.current_scene_path ?? null,
      });
    } catch (err) {
      console.error("Failed to fetch dirty state:", err);
    }
  },

  fetchComponentSchemas: async () => {
    const bridge = useRuntimeStore.getState().bridgeClient;
    if (!bridge || !bridge.isConnected()) return;

    set({ isLoadingSchemas: true });
    try {
      const res = await bridge.getComponentSchemas();
      const map: Record<string, ComponentSchema> = {};
      for (const s of res.schemas) {
        map[s.name] = s;
      }
      set({ componentSchemas: map, isLoadingSchemas: false });
    } catch (err) {
      console.error("Failed to fetch component schemas:", err);
      set({ isLoadingSchemas: false });
    }
  },

  undo: async () => {
    const bridge = useRuntimeStore.getState().bridgeClient;
    if (!bridge || !bridge.isConnected()) return;

    try {
      const res = await bridge.undo();
      if (res.success) {
        set({ canUndo: res.can_undo, canRedo: res.can_redo });
        await Promise.all([
          get().fetchHistory(),
          get().fetchDirty(),
          useSceneStore.getState().fetchSceneTree(),
        ]);
      }
    } catch (err) {
      console.error("Failed to undo:", err);
    }
  },

  redo: async () => {
    const bridge = useRuntimeStore.getState().bridgeClient;
    if (!bridge || !bridge.isConnected()) return;

    try {
      const res = await bridge.redo();
      if (res.success) {
        set({ canUndo: res.can_undo, canRedo: res.can_redo });
        await Promise.all([
          get().fetchHistory(),
          get().fetchDirty(),
          useSceneStore.getState().fetchSceneTree(),
        ]);
      }
    } catch (err) {
      console.error("Failed to redo:", err);
    }
  },

  jumpHistory: async (id: string) => {
    const bridge = useRuntimeStore.getState().bridgeClient;
    if (!bridge || !bridge.isConnected()) return;

    try {
      const res = await bridge.jumpHistory(id);
      if (res.success) {
        set({ canUndo: res.can_undo, canRedo: res.can_redo });
        await Promise.all([
          get().fetchHistory(),
          get().fetchDirty(),
          useSceneStore.getState().fetchSceneTree(),
        ]);
      }
    } catch (err) {
      console.error("Failed to jump history:", err);
    }
  },

  saveScene: async () => {
    const bridge = useRuntimeStore.getState().bridgeClient;
    if (!bridge || !bridge.isConnected()) return;

    try {
      const res = await bridge.saveScene();
      if (res.success) {
        set({ isDirty: false, scenePath: res.path });
        await get().fetchHistory();
      }
    } catch (err) {
      console.error("Failed to save scene:", err);
    }
  },

  setDirty: (dirty: boolean) => set({ isDirty: dirty }),
}));
