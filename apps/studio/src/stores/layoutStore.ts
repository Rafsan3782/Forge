import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import type { SerializedDockview } from "dockview-react";

export const STORAGE_KEY = "forge:studio:dock-layout-v1";

export interface LayoutState {
  layout: SerializedDockview | null;
  activePanel: string | null;
  isRestored: boolean;
  setLayout: (layout: SerializedDockview) => void;
  setActivePanel: (id: string | null) => void;
  resetLayout: () => void;
  clearPersistedLayout: () => void;
}

export const useLayoutStore = create<LayoutState>()(
  persist(
    (set) => ({
      layout: null,
      activePanel: null,
      isRestored: false,
      setLayout: (layout: SerializedDockview) => {
        set({ layout, isRestored: true });
      },
      setActivePanel: (id: string | null) => {
        set({ activePanel: id });
      },
      resetLayout: () => {
        set({ layout: null, isRestored: false });
        if (typeof window !== "undefined") {
          window.localStorage.removeItem(STORAGE_KEY);
        }
      },
      clearPersistedLayout: () => {
        if (typeof window !== "undefined") {
          window.localStorage.removeItem(STORAGE_KEY);
        }
      },
    }),
    {
      name: STORAGE_KEY,
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        layout: state.layout,
      }),
    },
  ),
);
