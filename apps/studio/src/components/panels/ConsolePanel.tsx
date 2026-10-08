import React, { useRef, useEffect } from "react";
import { Terminal, Trash2, CheckCircle2, AlertTriangle, AlertCircle, Info } from "lucide-react";
import { useLogStore } from "../../stores/logStore";

export const ConsolePanel: React.FC = () => {
  const { logs, clear } = useLogStore();
  const listEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    listEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs.length]);

  return (
    <div className="h-full w-full bg-[#121418] text-[#E8EAED] flex flex-col select-none text-xs font-mono">
      {/* Controls Bar */}
      <div className="h-7 px-3 bg-[#0A0B0D]/50 border-b border-[#23262D] flex items-center justify-between text-[11px] text-[#8B919C]">
        <div className="flex items-center gap-3">
          <span className="flex items-center gap-1.5 text-[#E8EAED] font-sans font-medium">
            <Terminal className="w-3 h-3 text-[#FF6A2B]" />
            <span>Output Log</span>
          </span>
          <span className="text-[#8B919C]">{logs.length} messages</span>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={clear}
            className="flex items-center gap-1 hover:text-[#E8EAED] transition-colors cursor-pointer"
            title="Clear Log"
          >
            <Trash2 className="w-3 h-3" />
            <span>Clear</span>
          </button>
        </div>
      </div>

      {/* Log Entries */}
      <div className="flex-1 overflow-y-auto p-2 space-y-1">
        {logs.map((log) => (
          <div
            key={log.id}
            className="flex items-start gap-2 px-1.5 py-0.5 rounded hover:bg-[#181B20] text-[11px] leading-relaxed"
          >
            <span className="text-[#8B919C] select-none shrink-0">{log.time}</span>
            {log.level === "success" && (
              <CheckCircle2 className="w-3.5 h-3.5 text-[#3DD68C] shrink-0 mt-0.5" />
            )}
            {log.level === "warn" && (
              <AlertTriangle className="w-3.5 h-3.5 text-[#F5A524] shrink-0 mt-0.5" />
            )}
            {log.level === "error" && (
              <AlertCircle className="w-3.5 h-3.5 text-[#FF4D4F] shrink-0 mt-0.5" />
            )}
            {log.level === "info" && (
              <Info className="w-3.5 h-3.5 text-[#7C5CFF] shrink-0 mt-0.5" />
            )}
            <span className="px-1 py-0.2 rounded bg-[#181B20] text-[#8B919C] text-[10px] shrink-0 font-mono">
              {log.source}
            </span>
            <span
              className={`break-all ${
                log.level === "error"
                  ? "text-[#FF4D4F] font-semibold"
                  : log.level === "warn"
                    ? "text-[#F5A524]"
                    : "text-[#E8EAED]"
              }`}
            >
              {log.message}
            </span>
          </div>
        ))}
        <div ref={listEndRef} />
      </div>
    </div>
  );
};
