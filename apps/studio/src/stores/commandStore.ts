import { create } from "zustand";

export interface CommandItem {
  id: string;
  title: string;
  category: "General" | "Panels" | "Engine" | "View";
  shortcut?: string;
  action: () => void;
}

export interface CommandState {
  isOpen: boolean;
  search: string;
  open: () => void;
  close: () => void;
  toggle: () => void;
  setSearch: (query: string) => void;
  commands: CommandItem[];
  registerCommand: (cmd: CommandItem) => void;
  unregisterCommand: (id: string) => void;
  executeCommand: (id: string) => void;
}

export const useCommandStore = create<CommandState>()((set, get) => ({
  isOpen: false,
  search: "",
  commands: [],
  open: () => set({ isOpen: true }),
  close: () => set({ isOpen: false, search: "" }),
  toggle: () => set((state) => ({ isOpen: !state.isOpen, search: "" })),
  setSearch: (search) => set({ search }),
  registerCommand: (cmd) => {
    set((state) => {
      const existing = state.commands.filter((c) => c.id !== cmd.id);
      return { commands: [...existing, cmd] };
    });
  },
  unregisterCommand: (id) => {
    set((state) => ({
      commands: state.commands.filter((c) => c.id !== id),
    }));
  },
  executeCommand: (id) => {
    const cmd = get().commands.find((c) => c.id === id);
    if (cmd) {
      cmd.action();
      get().close();
    }
  },
}));
