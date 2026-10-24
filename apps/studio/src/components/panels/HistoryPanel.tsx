import React, { useEffect } from "react";
import { History, RotateCcw, RotateCw, User, Bot } from "lucide-react";
import { useEditStore } from "../../stores/editStore";
import { useRuntimeStore } from "../../stores/runtimeStore";

export const HistoryPanel: React.FC = () => {
  const { undoStack, redoStack, canUndo, canRedo, fetchHistory, undo, redo, jumpHistory } =
    useEditStore();
  const { bridgeState, bridgeClient } = useRuntimeStore();

  useEffect(() => {
    if (bridgeState === "connected") {
      fetchHistory();
      const interval = setInterval(fetchHistory, 2000);
      return () => clearInterval(interval);
    }
  }, [bridgeState, bridgeClient, fetchHistory]);

  const formatTimestamp = (ts: number) => {
    if (!ts) return "";
    const d = new Date(ts * 1000);
    return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
  };

  return (
    <div className="h-full w-full bg-[#121418] text-[#E8EAED] flex flex-col select-none text-xs">
      {/* Header */}
      <div className="p-2 border-b border-[#23262D] flex items-center justify-between bg-[#0A0B0D]/50">
        <div className="flex items-center gap-1.5 font-medium text-[#8B919C]">
          <History className="w-3.5 h-3.5 text-[#FF6A2B]" />
          <span>Edit History</span>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={() => undo()}
            disabled={!canUndo}
            className={`p-1 rounded transition-colors ${
              canUndo
                ? "hover:bg-[#1C1E24] text-[#E8EAED] cursor-pointer"
                : "text-[#8B919C]/40 cursor-not-allowed"
            }`}
            title="Undo (Ctrl+Z)"
          >
            <RotateCcw className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={() => redo()}
            disabled={!canRedo}
            className={`p-1 rounded transition-colors ${
              canRedo
                ? "hover:bg-[#1C1E24] text-[#E8EAED] cursor-pointer"
                : "text-[#8B919C]/40 cursor-not-allowed"
            }`}
            title="Redo (Ctrl+Shift+Z)"
          >
            <RotateCw className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* History Items */}
      <div className="flex-1 overflow-y-auto p-2 space-y-1">
        {undoStack.length === 0 && redoStack.length === 0 ? (
          <div className="flex flex-col items-center justify-center p-6 text-center text-[#8B919C]">
            <History className="w-8 h-8 text-[#23262D] mb-2" />
            <span className="text-xs">No edit history yet</span>
            <span className="text-[11px] text-[#8B919C]/60 mt-1">
              Changes made by user or agents will appear here
            </span>
          </div>
        ) : (
          <>
            {/* Active Undo Stack */}
            {undoStack.map((entry, idx) => {
              const isLatest = idx === undoStack.length - 1;
              const isAgent = entry.author.toLowerCase() === "agent";

              return (
                <div
                  key={entry.id}
                  onClick={() => jumpHistory(entry.id)}
                  className={`p-2 rounded-md flex items-center justify-between cursor-pointer transition-colors border ${
                    isLatest
                      ? "bg-[#1C1E24] border-[#7C5CFF]/60 text-white shadow-sm"
                      : "bg-[#181B20]/40 border-transparent hover:bg-[#1C1E24]/60 text-[#E8EAED]"
                  }`}
                >
                  <div className="flex items-center gap-2 overflow-hidden flex-1">
                    <span className="w-1.5 h-1.5 rounded-full bg-[#3DD68C] shrink-0" />
                    {isAgent ? (
                      <Bot className="w-3.5 h-3.5 text-[#7C5CFF] shrink-0" title="Agent Action" />
                    ) : (
                      <User className="w-3.5 h-3.5 text-[#FF6A2B] shrink-0" title="User Action" />
                    )}
                    <span className="font-medium text-[11px] truncate">{entry.description}</span>
                  </div>

                  <div className="flex items-center gap-2 shrink-0">
                    <span className="text-[9px] text-[#8B919C] font-mono">
                      {formatTimestamp(entry.timestamp)}
                    </span>
                    <span className="text-[9px] px-1 py-0.5 rounded bg-[#121418] text-[#8B919C] border border-[#23262D] uppercase font-mono">
                      {entry.author}
                    </span>
                  </div>
                </div>
              );
            })}

            {/* Redo Stack Items (Dimmed / Future states) */}
            {redoStack.map((entry) => {
              const isAgent = entry.author.toLowerCase() === "agent";
              return (
                <div
                  key={entry.id}
                  onClick={() => jumpHistory(entry.id)}
                  className="p-2 rounded-md flex items-center justify-between cursor-pointer transition-colors border border-dashed border-[#23262D] bg-[#121418]/50 opacity-50 hover:opacity-80 text-[#8B919C]"
                >
                  <div className="flex items-center gap-2 overflow-hidden flex-1">
                    <span className="w-1.5 h-1.5 rounded-full bg-[#8B919C]/40 shrink-0" />
                    {isAgent ? (
                      <Bot className="w-3.5 h-3.5 shrink-0" />
                    ) : (
                      <User className="w-3.5 h-3.5 shrink-0" />
                    )}
                    <span className="text-[11px] truncate italic">{entry.description}</span>
                  </div>

                  <div className="flex items-center gap-2 shrink-0">
                    <span className="text-[9px] font-mono">{formatTimestamp(entry.timestamp)}</span>
                    <span className="text-[9px] px-1 py-0.5 rounded bg-[#121418] border border-[#23262D] uppercase font-mono">
                      {entry.author}
                    </span>
                  </div>
                </div>
              );
            })}
          </>
        )}
      </div>
    </div>
  );
};
