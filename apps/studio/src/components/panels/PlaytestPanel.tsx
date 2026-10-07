import React from "react";
import { PlaySquare, BarChart3, Clock, Cpu, CheckCircle } from "lucide-react";

export const PlaytestPanel: React.FC = () => {
  return (
    <div className="h-full w-full bg-[#121418] text-[#E8EAED] flex flex-col select-none text-xs">
      <div className="p-2 border-b border-[#23262D] flex items-center justify-between bg-[#0A0B0D]/50">
        <div className="flex items-center gap-1.5 font-medium text-[#8B919C]">
          <PlaySquare className="w-3.5 h-3.5 text-[#3DD68C]" />
          <span>Headless Playtest Telemetry</span>
        </div>
        <div className="flex items-center gap-2">
          <button className="px-2 py-0.5 rounded bg-[#3DD68C] hover:bg-[#34B878] text-[#0A0B0D] font-medium text-[11px] transition-colors cursor-pointer">
            Run Test Suite
          </button>
        </div>
      </div>

      <div className="flex-1 p-3 grid grid-cols-4 gap-3 overflow-y-auto">
        {/* Metric 1 */}
        <div className="p-2.5 rounded-lg border border-[#23262D] bg-[#181B20]/40 flex flex-col justify-between">
          <div className="flex items-center justify-between text-[#8B919C]">
            <span className="text-[11px]">Framerate</span>
            <BarChart3 className="w-3.5 h-3.5 text-[#3DD68C]" />
          </div>
          <div className="text-xl font-bold font-mono text-[#E8EAED] mt-2">
            60.0 <span className="text-xs text-[#8B919C]">FPS</span>
          </div>
          <span className="text-[10px] text-[#3DD68C] mt-1 font-mono">16.6ms avg frame time</span>
        </div>

        {/* Metric 2 */}
        <div className="p-2.5 rounded-lg border border-[#23262D] bg-[#181B20]/40 flex flex-col justify-between">
          <div className="flex items-center justify-between text-[#8B919C]">
            <span className="text-[11px]">Simulation Step</span>
            <Clock className="w-3.5 h-3.5 text-[#7C5CFF]" />
          </div>
          <div className="text-xl font-bold font-mono text-[#E8EAED] mt-2">
            1,240 <span className="text-xs text-[#8B919C]">ticks</span>
          </div>
          <span className="text-[10px] text-[#8B919C] mt-1 font-mono">Fixed timestep 60Hz</span>
        </div>

        {/* Metric 3 */}
        <div className="p-2.5 rounded-lg border border-[#23262D] bg-[#181B20]/40 flex flex-col justify-between">
          <div className="flex items-center justify-between text-[#8B919C]">
            <span className="text-[11px]">Memory / VRAM</span>
            <Cpu className="w-3.5 h-3.5 text-[#F5A524]" />
          </div>
          <div className="text-xl font-bold font-mono text-[#E8EAED] mt-2">
            48.2 <span className="text-xs text-[#8B919C]">MB</span>
          </div>
          <span className="text-[10px] text-[#8B919C] mt-1 font-mono">1 texture buffer</span>
        </div>

        {/* Metric 4 */}
        <div className="p-2.5 rounded-lg border border-[#23262D] bg-[#181B20]/40 flex flex-col justify-between">
          <div className="flex items-center justify-between text-[#8B919C]">
            <span className="text-[11px]">Autonomous Checks</span>
            <CheckCircle className="w-3.5 h-3.5 text-[#3DD68C]" />
          </div>
          <div className="text-xl font-bold font-mono text-[#3DD68C] mt-2">
            PASS <span className="text-xs text-[#8B919C]">5/5</span>
          </div>
          <span className="text-[10px] text-[#3DD68C] mt-1 font-mono">Zero assertion failures</span>
        </div>
      </div>
    </div>
  );
};
