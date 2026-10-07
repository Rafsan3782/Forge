import { describe, it, expect, beforeEach } from "vitest";
import type { SerializedDockview } from "dockview-react";
import { useLayoutStore, STORAGE_KEY } from "../stores/layoutStore";

describe("layoutStore", () => {
  beforeEach(() => {
    localStorage.clear();
    useLayoutStore.getState().resetLayout();
  });

  it("initializes with default layout state", () => {
    const state = useLayoutStore.getState();
    expect(state.layout).toBeNull();
    expect(state.activePanel).toBeNull();
    expect(state.isRestored).toBe(false);
  });

  it("sets layout and updates isRestored flag", () => {
    const mockLayout = {
      grid: { root: { type: "branch", data: [] }, width: 800, height: 600 },
      panels: {
        viewport: { id: "viewport", view: { title: "Viewport" } },
      },
    } as unknown as SerializedDockview;

    useLayoutStore.getState().setLayout(mockLayout);

    const state = useLayoutStore.getState();
    expect(state.layout).toEqual(mockLayout);
    expect(state.isRestored).toBe(true);
  });

  it("updates activePanel correctly", () => {
    useLayoutStore.getState().setActivePanel("scene_tree");
    expect(useLayoutStore.getState().activePanel).toBe("scene_tree");

    useLayoutStore.getState().setActivePanel(null);
    expect(useLayoutStore.getState().activePanel).toBeNull();
  });

  it("resets layout and clears storage", () => {
    const mockLayout = { grid: {}, panels: {} } as unknown as SerializedDockview;
    useLayoutStore.getState().setLayout(mockLayout);
    expect(useLayoutStore.getState().layout).not.toBeNull();

    useLayoutStore.getState().resetLayout();

    const state = useLayoutStore.getState();
    expect(state.layout).toBeNull();
    expect(state.isRestored).toBe(false);
    expect(localStorage.getItem(STORAGE_KEY)).toBeNull();
  });
});
