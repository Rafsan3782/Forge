import React from "react";
import { Terminal, Filter, Trash2, CheckCircle2, AlertTriangle, Info } from "lucide-react";

export const ConsolePanel: React.FC = () => {
  const dummyLogs = [
    {
      level: "info",
      time: "00:21:18.042",
      source: "forge_runtime",
      message: "Running headless mode: scene='templates/hello/scene.json', resolution=800x600",
    },
    {
      level: "info",
      time: "00:21:18.615",
      source: "forge_render",
      message: "GPU Adapter initialized: name='NVIDIA GeForce RTX 2060', backend=Vulkan",
    },
    {
      level: "success",
      time: "00:21:18.939",
      source: "forge_bridge",
      message: 'FORGE_BRIDGE_READY {"port":65506, "token":"9fa47..."}',
    },
    {
      level: "info",
      time: "00:21:20.104",
      source: "forge_bridge",
      message: "RPC dispatch: engine.ping -> pong (latency: 0.8ms)",
    },
    {
      level: "info",
      time: "00:21:25.430",
      source: "forge_bridge",
      message: "RPC dispatch: scene.get_tree -> 6 entities returned",
    },
  ];

  return (
    <div className="h-full w-full bg-[#121418] text-[#E8EAED] flex flex-col select-none text-xs font-mono">
      {/* Controls Bar */}
      <div className="h-7 px-3 bg-[#0A0B0D]/50 border-b border-[#23262D] flex items-center justify-between text-[11px] text-[#8B919C]">
        <div className="flex items-center gap-3">
          <span className="flex items-center gap-1.5 text-[#E8EAED] font-sans font-medium">
            <Terminal className="w-3 h-3 text-[#FF6A2B]" />
            <span>Output Log</span>
          </span>
          <span className="text-[#3DD68C]">● All systems nominal</span>
        </div>

        <div className="flex items-center gap-2">
          <button className="flex items-center gap-1 hover:text-[#E8EAED] transition-colors">
            <Filter className="w-3 h-3" />
            <span>Filter</span>
          </button>
          <button className="flex items-center gap-1 hover:text-[#E8EAED] transition-colors">
            <Trash2 className="w-3 h-3" />
            <span>Clear</span>
          </button>
        </div>
      </div>

      {/* Log Entries */}
      <div className="flex-1 overflow-y-auto p-2 space-y-1">
        {dummyLogs.map((log, idx) => (
          <div
            key={idx}
            className="flex items-start gap-2 px-1.5 py-0.5 rounded hover:bg-[#181B20] text-[11px] leading-relaxed"
          >
            <span className="text-[#8B919C] select-none">{log.time}</span>
            {log.level === "success" && (
              <CheckCircle2 className="w-3.5 h-3.5 text-[#3DD68C] shrink-0 mt-0.5" />
            )}
            {log.level === "warn" && (
              <AlertTriangle className="w-3.5 h-3.5 text-[#F5A524] shrink-0 mt-0.5" />
            )}
            {log.level === "info" && (
              <Info className="w-3.5 h-3.5 text-[#7C5CFF] shrink-0 mt-0.5" />
            )}
            <span className="px-1 py-0.2 rounded bg-[#181B20] text-[#8B919C] text-[10px] shrink-0">
              {log.source}
            </span>
            <span className="text-[#E8EAED] break-all">{log.message}</span>
          </div>
        ))}
      </div>
    </div>
  );
};
