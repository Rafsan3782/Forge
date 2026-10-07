import React from "react";
import { FolderTree, Eye, ChevronRight, Box, Camera } from "lucide-react";

export const SceneTreePanel: React.FC = () => {
  const dummyEntities = [
    { id: "camera_main", name: "Main Camera", type: "Camera2D", icon: Camera },
    { id: "player_hero", name: "Hero", type: "Sprite + Transform2D", icon: Box },
    { id: "item_gem", name: "Magic Gem", type: "Sprite + Transform2D", icon: Box },
    { id: "env_rock", name: "Ancient Rock", type: "Sprite + Transform2D", icon: Box },
    { id: "env_tree_left", name: "Left Tree", type: "Sprite + Transform2D", icon: Box },
    { id: "env_tree_right", name: "Right Tree", type: "Sprite + Transform2D", icon: Box },
  ];

  return (
    <div className="h-full w-full bg-[#121418] text-[#E8EAED] flex flex-col select-none text-xs">
      <div className="p-2 border-b border-[#23262D] flex items-center justify-between bg-[#0A0B0D]/50">
        <div className="flex items-center gap-1.5 font-medium text-[#8B919C]">
          <FolderTree className="w-3.5 h-3.5 text-[#FF6A2B]" />
          <span>Entities (6)</span>
        </div>
        <span className="text-[10px] text-[#8B919C] font-mono">scene.json</span>
      </div>

      <div className="flex-1 overflow-y-auto p-1.5 space-y-0.5">
        {dummyEntities.map((ent, idx) => {
          const Icon = ent.icon;
          return (
            <div
              key={ent.id}
              className={`flex items-center justify-between px-2 py-1.5 rounded-md hover:bg-[#1C1E24] cursor-pointer transition-colors duration-100 ${
                idx === 1 ? "bg-[#20232B] text-white border-l-2 border-[#FF6A2B]" : "text-[#E8EAED]"
              }`}
            >
              <div className="flex items-center gap-2">
                <ChevronRight className="w-3 h-3 text-[#8B919C]" />
                <Icon className="w-3.5 h-3.5 text-[#8B919C]" />
                <span className="font-medium">{ent.name}</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-[10px] text-[#8B919C] font-mono">{ent.type}</span>
                <Eye className="w-3 h-3 text-[#8B919C] hover:text-[#E8EAED]" />
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
