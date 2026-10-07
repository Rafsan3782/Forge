import { describe, it, expect, beforeEach, vi } from "vitest";
import { useCommandStore } from "../stores/commandStore";

describe("commandStore", () => {
  beforeEach(() => {
    useCommandStore.setState({
      isOpen: false,
      search: "",
      commands: [],
    });
  });

  it("handles opening, closing, and toggling state", () => {
    const store = useCommandStore.getState();
    expect(store.isOpen).toBe(false);

    store.open();
    expect(useCommandStore.getState().isOpen).toBe(true);

    store.close();
    expect(useCommandStore.getState().isOpen).toBe(false);

    store.toggle();
    expect(useCommandStore.getState().isOpen).toBe(true);

    store.toggle();
    expect(useCommandStore.getState().isOpen).toBe(false);
  });

  it("registers and executes commands", () => {
    const actionSpy = vi.fn();
    const testCmd = {
      id: "test.reset",
      title: "Reset Workspace",
      category: "General" as const,
      shortcut: "Ctrl+R",
      action: actionSpy,
    };

    useCommandStore.getState().registerCommand(testCmd);
    expect(useCommandStore.getState().commands).toHaveLength(1);

    useCommandStore.getState().open();
    useCommandStore.getState().executeCommand("test.reset");

    expect(actionSpy).toHaveBeenCalledOnce();
    expect(useCommandStore.getState().isOpen).toBe(false);
  });

  it("unregisters commands by id", () => {
    const cmd1 = { id: "cmd1", title: "One", category: "General" as const, action: () => {} };
    const cmd2 = { id: "cmd2", title: "Two", category: "Panels" as const, action: () => {} };

    useCommandStore.getState().registerCommand(cmd1);
    useCommandStore.getState().registerCommand(cmd2);
    expect(useCommandStore.getState().commands).toHaveLength(2);

    useCommandStore.getState().unregisterCommand("cmd1");
    expect(useCommandStore.getState().commands).toHaveLength(1);
    expect(useCommandStore.getState().commands[0].id).toBe("cmd2");
  });
});
