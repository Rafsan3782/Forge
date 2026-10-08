import React, { useEffect } from "react";
import { FolderTree, ChevronRight, Box, Camera, RefreshCw } from "lucide-react";
import { useSceneStore } from "../../stores/sceneStore";
import { useRuntimeStore } from "../../stores/runtimeStore";
import type { EntityNode } from "@forge/protocol";

export const SceneTreePanel: React.FC = () => {
  const { entities, selectedEntityId, isLoading, fetchSceneTree, selectEntity } = useSceneStore();
  const { bridgeState, bridgeClient } = useRuntimeStore();

  useEffect(() => {
    if (bridgeState === "connected") {
      fetchSceneTree();
      const interval = setInterval(fetchSceneTree, 2500);
      return () => clearInterval(interval);
    }
  }, [bridgeState, bridgeClient, fetchSceneTree]);

  const renderNode = (node: EntityNode, depth = 0) => {
    const isSelected = selectedEntityId === node.id;
    const isCamera = node.name.toLowerCase().includes("camera");
    const Icon = isCamera ? Camera : Box;

    return (
      <div key={node.id} className="space-y-0.5">
        <div
          onClick={() => selectEntity(node.id)}
          style={{ paddingLeft: `${depth * 14 + 8}px` }}
          className={`flex items-center justify-between pr-2 py-1.5 rounded-md hover:bg-[#1C1E24] cursor-pointer transition-colors duration-100 ${
            isSelected ? "bg-[#20232B] text-white border-l-2 border-[#FF6A2B]" : "text-[#E8EAED]"
          }`}
        >
          <div className="flex items-center gap-2 overflow-hidden">
            <ChevronRight className="w-3 h-3 text-[#8B919C] shrink-0" />
            <Icon className="w-3.5 h-3.5 text-[#8B919C] shrink-0" />
            <span className="font-medium truncate">{node.name}</span>
          </div>
          <div className="flex items-center gap-1.5 shrink-0">
            <span className="text-[10px] text-[#8B919C] font-mono">
              {node.components.length} comps
            </span>
          </div>
        </div>

        {node.children && node.children.map((child) => renderNode(child, depth + 1))}
      </div>
    );
  };

  return (
    <div className="h-full w-full bg-[#121418] text-[#E8EAED] flex flex-col select-none text-xs">
      <div className="p-2 border-b border-[#23262D] flex items-center justify-between bg-[#0A0B0D]/50">
        <div className="flex items-center gap-1.5 font-medium text-[#8B919C]">
          <FolderTree className="w-3.5 h-3.5 text-[#FF6A2B]" />
          <span>Entities ({entities.length})</span>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => fetchSceneTree()}
            className="p-1 rounded hover:bg-[#1C1E24] text-[#8B919C] hover:text-[#E8EAED] transition-colors cursor-pointer"
            title="Refresh Scene Tree"
          >
            <RefreshCw className={`w-3 h-3 ${isLoading ? "animate-spin" : ""}`} />
          </button>
          <span className="text-[10px] text-[#8B919C] font-mono">scene.json</span>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-1.5 space-y-0.5">
        {entities.length === 0 ? (
          <div className="flex flex-col items-center justify-center p-6 text-center text-[#8B919C]">
            <span className="text-xs">No live entities connected</span>
            <span className="text-[11px] text-[#8B919C]/60 mt-1 font-mono">
              Launch runtime to inspect ECS tree
            </span>
          </div>
        ) : (
          entities.map((node) => renderNode(node))
        )}
      </div>
    </div>
  );
};
