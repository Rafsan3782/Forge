import { create } from "zustand";
import type { EntityNode } from "@forge/protocol";
import { useRuntimeStore } from "./runtimeStore";

export interface SceneStoreState {
  entities: EntityNode[];
  selectedEntityId: string | null;
  selectedEntityIds: string[];
  selectedEntity: EntityNode | null;
  isLoading: boolean;
  error: string | null;

  fetchSceneTree: () => Promise<void>;
  selectEntity: (id: string | null, multi?: boolean) => void;
  createEntity: (name?: string, parentId?: string | null) => Promise<void>;
  deleteEntity: (id: string) => Promise<void>;
  duplicateEntity: (id: string) => Promise<void>;
  reparentEntity: (id: string, newParentId: string | null) => Promise<void>;
  renameEntity: (id: string, newName: string) => Promise<void>;
}

function findEntity(nodes: EntityNode[], id: string): EntityNode | null {
  for (const node of nodes) {
    if (node.id === id) return node;
    const found = findEntity(node.children, id);
    if (found) return found;
  }
  return null;
}

export const useSceneStore = create<SceneStoreState>((set, get) => ({
  entities: [],
  selectedEntityId: null,
  selectedEntityIds: [],
  selectedEntity: null,
  isLoading: false,
  error: null,

  fetchSceneTree: async () => {
    const bridge = useRuntimeStore.getState().bridgeClient;
    if (!bridge || !bridge.isConnected()) {
      return;
    }

    set({ isLoading: true, error: null });
    try {
      const res = await bridge.getSceneTree();
      const currentSelectedId = get().selectedEntityId;
      const selected = currentSelectedId ? findEntity(res.root_entities, currentSelectedId) : null;

      set({
        entities: res.root_entities,
        selectedEntity: selected || (res.root_entities.length > 0 ? res.root_entities[0] : null),
        selectedEntityId: selected ? currentSelectedId : (res.root_entities[0]?.id ?? null),
        selectedEntityIds: selected
          ? get().selectedEntityIds
          : res.root_entities[0]
            ? [res.root_entities[0].id]
            : [],
        isLoading: false,
      });
    } catch (err) {
      set({ isLoading: false, error: String(err) });
    }
  },

  selectEntity: (id: string | null, multi = false) => {
    if (!id) {
      set({ selectedEntityId: null, selectedEntityIds: [], selectedEntity: null });
      return;
    }
    const found = findEntity(get().entities, id);
    if (multi) {
      const currentIds = get().selectedEntityIds;
      const nextIds = currentIds.includes(id)
        ? currentIds.filter((item) => item !== id)
        : [...currentIds, id];
      set({
        selectedEntityId: id,
        selectedEntityIds: nextIds,
        selectedEntity: found,
      });
    } else {
      set({
        selectedEntityId: id,
        selectedEntityIds: [id],
        selectedEntity: found,
      });
    }
  },

  createEntity: async (name = "New Entity", parentId = null) => {
    const bridge = useRuntimeStore.getState().bridgeClient;
    if (!bridge || !bridge.isConnected()) return;

    try {
      const res = await bridge.createEntity(name, parentId);
      await get().fetchSceneTree();
      get().selectEntity(res.entity_id);
    } catch (err) {
      console.error("Failed to create entity:", err);
    }
  },

  deleteEntity: async (id: string) => {
    const bridge = useRuntimeStore.getState().bridgeClient;
    if (!bridge || !bridge.isConnected()) return;

    try {
      await bridge.deleteEntity(id);
      await get().fetchSceneTree();
      if (get().selectedEntityId === id) {
        get().selectEntity(null);
      }
    } catch (err) {
      console.error("Failed to delete entity:", err);
    }
  },

  duplicateEntity: async (id: string) => {
    const bridge = useRuntimeStore.getState().bridgeClient;
    if (!bridge || !bridge.isConnected()) return;

    try {
      const res = await bridge.duplicateEntity(id);
      await get().fetchSceneTree();
      get().selectEntity(res.new_entity_id);
    } catch (err) {
      console.error("Failed to duplicate entity:", err);
    }
  },

  reparentEntity: async (id: string, newParentId: string | null) => {
    const bridge = useRuntimeStore.getState().bridgeClient;
    if (!bridge || !bridge.isConnected()) return;

    try {
      await bridge.reparentEntity(id, newParentId);
      await get().fetchSceneTree();
    } catch (err) {
      console.error("Failed to reparent entity:", err);
    }
  },

  renameEntity: async (id: string, newName: string) => {
    const bridge = useRuntimeStore.getState().bridgeClient;
    if (!bridge || !bridge.isConnected()) return;

    try {
      await bridge.setComponent(id, "Name", { name: newName });
      await get().fetchSceneTree();
    } catch (err) {
      console.error("Failed to rename entity:", err);
    }
  },
}));
