import React from "react";
import { Sliders, Plus, Hash, Move } from "lucide-react";

export const InspectorPanel: React.FC = () => {
  return (
    <div className="h-full w-full bg-[#121418] text-[#E8EAED] flex flex-col select-none text-xs">
      {/* Header */}
      <div className="p-2 border-b border-[#23262D] flex items-center justify-between bg-[#0A0B0D]/50">
        <div className="flex items-center gap-1.5 font-medium text-[#8B919C]">
          <Sliders className="w-3.5 h-3.5 text-[#FF6A2B]" />
          <span>Inspector</span>
        </div>
        <span className="text-[10px] text-[#3DD68C] font-mono">player_hero</span>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-3 space-y-4">
        {/* Entity Metadata */}
        <div className="space-y-1.5">
          <label className="text-[11px] font-semibold text-[#8B919C] uppercase tracking-wider">
            Entity
          </label>
          <div className="flex items-center gap-2">
            <span className="w-16 text-[#8B919C]">Name</span>
            <input
              type="text"
              readOnly
              value="Hero"
              className="flex-1 px-2 py-1 bg-[#181B20] border border-[#23262D] rounded text-[#E8EAED] font-medium outline-none"
            />
          </div>
          <div className="flex items-center gap-2">
            <span className="w-16 text-[#8B919C]">ID</span>
            <input
              type="text"
              readOnly
              value="player_hero"
              className="flex-1 px-2 py-1 bg-[#181B20] border border-[#23262D] rounded text-[#8B919C] font-mono outline-none"
            />
          </div>
        </div>

        {/* Transform2D Component */}
        <div className="p-2.5 rounded-lg border border-[#23262D] bg-[#181B20]/40 space-y-2.5">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-1.5 font-medium text-[#E8EAED]">
              <Move className="w-3.5 h-3.5 text-[#7C5CFF]" />
              <span>Transform2D</span>
            </div>
            <span className="text-[10px] text-[#8B919C] font-mono">Component</span>
          </div>

          <div className="grid grid-cols-2 gap-2 text-[11px]">
            <div>
              <span className="text-[#8B919C] block mb-1">Position (X, Y)</span>
              <div className="flex gap-1 font-mono">
                <input
                  readOnly
                  value="0.0"
                  className="w-full px-1.5 py-0.5 bg-[#121418] border border-[#23262D] rounded text-center text-[#E8EAED]"
                />
                <input
                  readOnly
                  value="0.0"
                  className="w-full px-1.5 py-0.5 bg-[#121418] border border-[#23262D] rounded text-center text-[#E8EAED]"
                />
              </div>
            </div>
            <div>
              <span className="text-[#8B919C] block mb-1">Scale (X, Y)</span>
              <div className="flex gap-1 font-mono">
                <input
                  readOnly
                  value="1.0"
                  className="w-full px-1.5 py-0.5 bg-[#121418] border border-[#23262D] rounded text-center text-[#E8EAED]"
                />
                <input
                  readOnly
                  value="1.0"
                  className="w-full px-1.5 py-0.5 bg-[#121418] border border-[#23262D] rounded text-center text-[#E8EAED]"
                />
              </div>
            </div>
          </div>
        </div>

        {/* Sprite Component */}
        <div className="p-2.5 rounded-lg border border-[#23262D] bg-[#181B20]/40 space-y-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-1.5 font-medium text-[#E8EAED]">
              <Hash className="w-3.5 h-3.5 text-[#FF6A2B]" />
              <span>Sprite</span>
            </div>
            <span className="text-[10px] text-[#8B919C] font-mono">Component</span>
          </div>
          <div className="text-[11px] space-y-1">
            <div className="flex justify-between">
              <span className="text-[#8B919C]">Texture</span>
              <span className="font-mono text-[#E8EAED]">assets/hero.png</span>
            </div>
            <div className="flex justify-between">
              <span className="text-[#8B919C]">Size</span>
              <span className="font-mono text-[#E8EAED]">64 × 64 px</span>
            </div>
          </div>
        </div>

        {/* Add Component Button */}
        <button className="w-full py-1.5 rounded-md border border-dashed border-[#23262D] hover:border-[#7C5CFF] text-[#8B919C] hover:text-[#E8EAED] flex items-center justify-center gap-1.5 transition-colors cursor-pointer">
          <Plus className="w-3.5 h-3.5" />
          <span>Add Component</span>
        </button>
      </div>
    </div>
  );
};
