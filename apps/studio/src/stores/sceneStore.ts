import { create } from "zustand";
import type { EntityNode } from "@forge/protocol";
import { useRuntimeStore } from "./runtimeStore";

export interface SceneStoreState {
  entities: EntityNode[];
  selectedEntityId: string | null;
  selectedEntity: EntityNode | null;
  isLoading: boolean;
  error: string | null;

  fetchSceneTree: () => Promise<void>;
  selectEntity: (id: string | null) => void;
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
        isLoading: false,
      });
    } catch (err) {
      set({ isLoading: false, error: String(err) });
    }
  },

  selectEntity: (id: string | null) => {
    if (!id) {
      set({ selectedEntityId: null, selectedEntity: null });
      return;
    }
    const found = findEntity(get().entities, id);
    set({ selectedEntityId: id, selectedEntity: found });
  },
}));
