import React from "react";
import { RefreshCw } from "lucide-react";
import { useRuntimeStore } from "../stores/runtimeStore";
import { useSceneStore } from "../stores/sceneStore";

export const StatusBar: React.FC = () => {
  const { status, port, mode, bridgeState, fps, restartRuntime } = useRuntimeStore();
  const { entities } = useSceneStore();

  const getStatusColor = () => {
    if (status === "crashed") return "bg-[#FF4D4F]";
    if (bridgeState === "connected") return "bg-[#3DD68C]";
    if (status === "starting" || bridgeState === "connecting") return "bg-[#F5A524]";
    return "bg-[#8B919C]";
  };

  const getStatusLabel = () => {
    if (status === "crashed") return "Engine Crashed";
    if (bridgeState === "connected") return `Bridge Connected (127.0.0.1:${port})`;
    if (status === "starting") return "Starting Runtime...";
    if (bridgeState === "connecting") return "Connecting Bridge...";
    return "Engine Stopped";
  };

  return (
    <footer className="h-6 w-full bg-[#0A0B0D] border-t border-[#23262D] flex items-center justify-between px-3 select-none text-[11px] font-mono text-[#8B919C] z-40 shrink-0">
      {/* Left: Connection State & Quick Toggle */}
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-1.5">
          <span
            className={`w-2 h-2 rounded-full ${getStatusColor()} ${status === "running" ? "animate-pulse" : ""}`}
          />
          <span className="text-[#E8EAED] font-sans font-medium">{getStatusLabel()}</span>
        </div>

        <div className="h-3 w-[1px] bg-[#23262D]" />

        <div className="flex items-center gap-1 text-[10px]">
          <span className="text-[#8B919C]">Mode:</span>
          <span className={mode === "embed" ? "text-[#3DD68C]" : "text-[#7C5CFF]"}>
            {mode === "embed" ? "HWND Child" : "Stream Fallback"}
          </span>
        </div>

        {status === "crashed" && (
          <button
            onClick={() => restartRuntime()}
            className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-[#FF4D4F]/20 text-[#FF4D4F] hover:bg-[#FF4D4F]/30 border border-[#FF4D4F]/40 cursor-pointer"
          >
            <RefreshCw className="w-2.5 h-2.5" />
            <span>Restart</span>
          </button>
        )}
      </div>

      {/* Right: Scene entities & frame rate */}
      <div className="flex items-center gap-3 text-[10px]">
        {status === "running" && (
          <>
            <span className="text-[#8B919C]">{entities.length} ECS Entities</span>
            <div className="h-3 w-[1px] bg-[#23262D]" />
            <span className="text-[#3DD68C]">{fps.toFixed(1)} FPS</span>
            <div className="h-3 w-[1px] bg-[#23262D]" />
          </>
        )}
        <span className="text-[#8B919C]/60">Forge Studio v0.1.0</span>
      </div>
    </footer>
  );
};
