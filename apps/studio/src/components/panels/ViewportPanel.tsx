import React from "react";
import { Play, Pause, RotateCcw, Maximize2, Crosshair } from "lucide-react";

export const ViewportPanel: React.FC = () => {
  return (
    <div className="h-full w-full bg-[#0A0B0D] flex flex-col select-none relative overflow-hidden">
      {/* Top Overlay Controls Bar */}
      <div className="h-9 px-3 bg-[#121418]/80 backdrop-blur-xs border-b border-[#23262D] flex items-center justify-between text-xs z-10">
        <div className="flex items-center gap-1.5">
          <button className="flex items-center gap-1 px-2 py-1 rounded bg-[#FF6A2B] hover:bg-[#E05A20] text-white font-medium text-xs transition-colors duration-150 cursor-pointer">
            <Play className="w-3 h-3 fill-white" />
            <span>Play</span>
          </button>
          <button className="p-1 rounded hover:bg-[#1C1E24] text-[#8B919C] hover:text-[#E8EAED] transition-colors duration-150">
            <Pause className="w-3.5 h-3.5" />
          </button>
          <button className="p-1 rounded hover:bg-[#1C1E24] text-[#8B919C] hover:text-[#E8EAED] transition-colors duration-150">
            <RotateCcw className="w-3.5 h-3.5" />
          </button>
        </div>

        <div className="flex items-center gap-3 text-[#8B919C] text-[11px] font-mono">
          <span className="flex items-center gap-1">
            <span className="w-2 h-2 rounded-full bg-[#3DD68C] animate-pulse" />
            <span>60.0 FPS</span>
          </span>
          <span>800 × 600 (100%)</span>
          <button className="p-1 rounded hover:bg-[#1C1E24] hover:text-[#E8EAED] transition-colors">
            <Maximize2 className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Viewport Canvas Grid Placeholder */}
      <div className="flex-1 flex items-center justify-center relative bg-[radial-gradient(#23262D_1px,transparent_1px)] [background-size:24px_24px]">
        {/* Game Canvas Boundary Mock */}
        <div className="w-[520px] h-[390px] border border-[#23262D] rounded bg-[#121418]/40 shadow-2xl flex flex-col items-center justify-center relative">
          <Crosshair className="w-8 h-8 text-[#7C5CFF]/60 mb-3" />
          <span className="text-sm font-semibold text-[#E8EAED] tracking-tight">
            WGPU Engine 2D Viewport
          </span>
          <span className="text-xs text-[#8B919C] mt-1 font-mono">
            Camera: (0.0, 0.0) | Zoom: 1.0x
          </span>
          <div className="mt-4 px-2.5 py-1 rounded-full bg-[#181B20] border border-[#23262D] text-[11px] text-[#8B919C] font-mono">
            Bridge Connected (127.0.0.1)
          </div>
        </div>
      </div>
    </div>
  );
};
