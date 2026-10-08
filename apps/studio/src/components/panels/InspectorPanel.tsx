import React from "react";
import { Sliders, Move, Hash, Camera, Shield, Cpu } from "lucide-react";
import { useSceneStore } from "../../stores/sceneStore";

export const InspectorPanel: React.FC = () => {
  const { selectedEntity } = useSceneStore();

  if (!selectedEntity) {
    return (
      <div className="h-full w-full bg-[#121418] text-[#E8EAED] flex flex-col select-none text-xs">
        <div className="p-2 border-b border-[#23262D] flex items-center justify-between bg-[#0A0B0D]/50">
          <div className="flex items-center gap-1.5 font-medium text-[#8B919C]">
            <Sliders className="w-3.5 h-3.5 text-[#FF6A2B]" />
            <span>Inspector</span>
          </div>
        </div>
        <div className="flex-1 flex flex-col items-center justify-center p-6 text-center text-[#8B919C]">
          <Sliders className="w-8 h-8 text-[#23262D] mb-2" />
          <span className="text-xs font-medium">No Entity Selected</span>
          <span className="text-[11px] text-[#8B919C]/60 mt-1">
            Select an entity in the Scene Tree to inspect its components
          </span>
        </div>
      </div>
    );
  }

  const getComponentIcon = (name: string) => {
    const lower = name.toLowerCase();
    if (lower.includes("transform")) return Move;
    if (lower.includes("sprite")) return Hash;
    if (lower.includes("camera")) return Camera;
    if (lower.includes("collider") || lower.includes("body")) return Shield;
    return Cpu;
  };

  return (
    <div className="h-full w-full bg-[#121418] text-[#E8EAED] flex flex-col select-none text-xs">
      {/* Header */}
      <div className="p-2 border-b border-[#23262D] flex items-center justify-between bg-[#0A0B0D]/50">
        <div className="flex items-center gap-1.5 font-medium text-[#8B919C]">
          <Sliders className="w-3.5 h-3.5 text-[#FF6A2B]" />
          <span>Inspector</span>
        </div>
        <span className="text-[10px] text-[#3DD68C] font-mono truncate max-w-[140px]">
          {selectedEntity.id}
        </span>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-3 space-y-4">
        {/* Entity Metadata */}
        <div className="space-y-1.5">
          <label className="text-[11px] font-semibold text-[#8B919C] uppercase tracking-wider">
            Entity Info
          </label>
          <div className="flex items-center gap-2">
            <span className="w-14 text-[#8B919C]">Name</span>
            <input
              type="text"
              readOnly
              value={selectedEntity.name}
              className="flex-1 px-2 py-1 bg-[#181B20] border border-[#23262D] rounded text-[#E8EAED] font-medium outline-none"
            />
          </div>
          <div className="flex items-center gap-2">
            <span className="w-14 text-[#8B919C]">ID</span>
            <input
              type="text"
              readOnly
              value={selectedEntity.id}
              className="flex-1 px-2 py-1 bg-[#181B20] border border-[#23262D] rounded text-[#8B919C] font-mono outline-none text-[11px]"
            />
          </div>
        </div>

        {/* Dynamic Components List */}
        <div className="space-y-2.5">
          <label className="text-[11px] font-semibold text-[#8B919C] uppercase tracking-wider">
            Components ({selectedEntity.components.length})
          </label>

          {selectedEntity.components.map((compName) => {
            const Icon = getComponentIcon(compName);
            return (
              <div
                key={compName}
                className="p-2.5 rounded-lg border border-[#23262D] bg-[#181B20]/50 space-y-2"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-1.5 font-medium text-[#E8EAED]">
                    <Icon className="w-3.5 h-3.5 text-[#7C5CFF]" />
                    <span>{compName}</span>
                  </div>
                  <span className="text-[10px] text-[#8B919C] font-mono">ECS Component</span>
                </div>

                {compName.toLowerCase().includes("transform") && (
                  <div className="grid grid-cols-2 gap-2 text-[11px] pt-1">
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
                )}

                {compName.toLowerCase().includes("sprite") && (
                  <div className="text-[11px] space-y-1 pt-1">
                    <div className="flex justify-between">
                      <span className="text-[#8B919C]">Rendering</span>
                      <span className="font-mono text-[#E8EAED]">Renderer2D Quad</span>
                    </div>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
