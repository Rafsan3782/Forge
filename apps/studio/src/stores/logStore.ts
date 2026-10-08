import { create } from "zustand";

export interface LogEntry {
  id: string;
  time: string;
  level: "info" | "warn" | "error" | "success";
  source: string;
  message: string;
}

interface LogStore {
  logs: LogEntry[];
  addLog: (entry: Omit<LogEntry, "id" | "time">) => void;
  clear: () => void;
}

function formatCurrentTime(): string {
  const now = new Date();
  const pad = (n: number, z = 2) => n.toString().padStart(z, "0");
  return `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds())}.${pad(now.getMilliseconds(), 3)}`;
}

export const useLogStore = create<LogStore>((set) => ({
  logs: [
    {
      id: "init-1",
      time: formatCurrentTime(),
      level: "info",
      source: "studio",
      message: "Forge Studio initialized. Ready for engine lifecycle events.",
    },
  ],
  addLog: (entry) =>
    set((state) => ({
      logs: [
        ...state.logs.slice(-499), // retain last 500 lines
        {
          ...entry,
          id: Math.random().toString(36).substring(2, 9),
          time: formatCurrentTime(),
        },
      ],
    })),
  clear: () => set({ logs: [] }),
}));
