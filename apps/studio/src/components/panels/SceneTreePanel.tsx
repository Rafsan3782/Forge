import React, { useEffect, useState } from "react";
import {
  FolderTree,
  ChevronRight,
  ChevronDown,
  Box,
  Camera,
  RefreshCw,
  Plus,
  Trash2,
  Copy,
  Edit2,
} from "lucide-react";
import { useSceneStore } from "../../stores/sceneStore";
import { useRuntimeStore } from "../../stores/runtimeStore";
import type { EntityNode } from "@forge/protocol";

export const SceneTreePanel: React.FC = () => {
  const {
    entities,
    selectedEntityId,
    selectedEntityIds,
    isLoading,
    fetchSceneTree,
    selectEntity,
    createEntity,
    deleteEntity,
    duplicateEntity,
    reparentEntity,
    renameEntity,
  } = useSceneStore();
  const { bridgeState, bridgeClient } = useRuntimeStore();

  const [expandedNodes, setExpandedNodes] = useState<Record<string, boolean>>({});
  const [editingNodeId, setEditingNodeId] = useState<string | null>(null);
  const [editingName, setEditingName] = useState("");
  const [draggedNodeId, setDraggedNodeId] = useState<string | null>(null);

  useEffect(() => {
    if (bridgeState === "connected") {
      fetchSceneTree();
      const interval = setInterval(fetchSceneTree, 2500);
      return () => clearInterval(interval);
    }
  }, [bridgeState, bridgeClient, fetchSceneTree]);

  // Keyboard shortcuts (F2 rename, Ctrl+D duplicate, Delete)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.target as HTMLElement).tagName === "INPUT") return;

      if (e.key === "F2" && selectedEntityId) {
        e.preventDefault();
        const node = findNodeById(entities, selectedEntityId);
        if (node) {
          setEditingNodeId(node.id);
          setEditingName(node.name);
        }
      } else if (e.ctrlKey && e.key === "d" && selectedEntityId) {
        e.preventDefault();
        duplicateEntity(selectedEntityId);
      } else if ((e.key === "Delete" || e.key === "Backspace") && selectedEntityId) {
        e.preventDefault();
        deleteEntity(selectedEntityId);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [selectedEntityId, entities, duplicateEntity, deleteEntity]);

  const findNodeById = (nodes: EntityNode[], id: string): EntityNode | null => {
    for (const n of nodes) {
      if (n.id === id) return n;
      const f = findNodeById(n.children, id);
      if (f) return f;
    }
    return null;
  };

  const toggleExpand = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setExpandedNodes((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  const commitRename = (id: string) => {
    if (editingName.trim()) {
      renameEntity(id, editingName.trim());
    }
    setEditingNodeId(null);
  };

  const handleDragStart = (id: string, e: React.DragEvent) => {
    e.stopPropagation();
    setDraggedNodeId(id);
    e.dataTransfer.setData("text/plain", id);
  };

  const handleDrop = (targetId: string, e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    const sourceId = draggedNodeId || e.dataTransfer.getData("text/plain");
    if (sourceId && sourceId !== targetId) {
      reparentEntity(sourceId, targetId);
    }
    setDraggedNodeId(null);
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
  };

  const renderNode = (node: EntityNode, depth = 0) => {
    const isSelected = selectedEntityIds.includes(node.id);
    const isEditing = editingNodeId === node.id;
    const isCamera = node.name.toLowerCase().includes("camera");
    const hasChildren = node.children && node.children.length > 0;
    const isExpanded = expandedNodes[node.id] ?? true;
    const Icon = isCamera ? Camera : Box;

    return (
      <div key={node.id} className="space-y-0.5">
        <div
          draggable={!isEditing}
          onDragStart={(e) => handleDragStart(node.id, e)}
          onDragOver={handleDragOver}
          onDrop={(e) => handleDrop(node.id, e)}
          onClick={(e) => selectEntity(node.id, e.ctrlKey || e.metaKey)}
          style={{ paddingLeft: `${depth * 14 + 8}px` }}
          className={`flex items-center justify-between pr-2 py-1.5 rounded-md hover:bg-[#1C1E24] cursor-pointer transition-colors duration-100 group ${
            isSelected ? "bg-[#20232B] text-white border-l-2 border-[#FF6A2B]" : "text-[#E8EAED]"
          }`}
        >
          <div className="flex items-center gap-1.5 overflow-hidden flex-1">
            {hasChildren ? (
              <button
                onClick={(e) => toggleExpand(node.id, e)}
                className="p-0.5 text-[#8B919C] hover:text-white"
              >
                {isExpanded ? (
                  <ChevronDown className="w-3 h-3 shrink-0" />
                ) : (
                  <ChevronRight className="w-3 h-3 shrink-0" />
                )}
              </button>
            ) : (
              <span className="w-4 shrink-0" />
            )}
            <Icon className="w-3.5 h-3.5 text-[#8B919C] shrink-0" />

            {isEditing ? (
              <input
                type="text"
                value={editingName}
                autoFocus
                onChange={(e) => setEditingName(e.target.value)}
                onBlur={() => commitRename(node.id)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") commitRename(node.id);
                  if (e.key === "Escape") setEditingNodeId(null);
                }}
                className="bg-[#121418] border border-[#7C5CFF] rounded px-1 text-[11px] text-white outline-none w-32"
              />
            ) : (
              <span
                onDoubleClick={() => {
                  setEditingNodeId(node.id);
                  setEditingName(node.name);
                }}
                className="font-medium truncate text-[11px]"
              >
                {node.name}
              </span>
            )}
          </div>

          <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              onClick={(e) => {
                e.stopPropagation();
                duplicateEntity(node.id);
              }}
              className="p-1 rounded hover:bg-[#2A2E37] text-[#8B919C] hover:text-white"
              title="Duplicate (Ctrl+D)"
            >
              <Copy className="w-3 h-3" />
            </button>
            <button
              onClick={(e) => {
                e.stopPropagation();
                setEditingNodeId(node.id);
                setEditingName(node.name);
              }}
              className="p-1 rounded hover:bg-[#2A2E37] text-[#8B919C] hover:text-white"
              title="Rename (F2)"
            >
              <Edit2 className="w-3 h-3" />
            </button>
            <button
              onClick={(e) => {
                e.stopPropagation();
                deleteEntity(node.id);
              }}
              className="p-1 rounded hover:bg-[#2A2E37] text-[#8B919C] hover:text-[#FF4D4F]"
              title="Delete"
            >
              <Trash2 className="w-3 h-3" />
            </button>
          </div>
        </div>

        {hasChildren && isExpanded && node.children.map((child) => renderNode(child, depth + 1))}
      </div>
    );
  };

  return (
    <div
      onDragOver={handleDragOver}
      onDrop={(e) => {
        // Drop on root level unparents entity
        e.preventDefault();
        const sourceId = draggedNodeId || e.dataTransfer.getData("text/plain");
        if (sourceId) {
          reparentEntity(sourceId, null);
        }
        setDraggedNodeId(null);
      }}
      className="h-full w-full bg-[#121418] text-[#E8EAED] flex flex-col select-none text-xs"
    >
      <div className="p-2 border-b border-[#23262D] flex items-center justify-between bg-[#0A0B0D]/50">
        <div className="flex items-center gap-1.5 font-medium text-[#8B919C]">
          <FolderTree className="w-3.5 h-3.5 text-[#FF6A2B]" />
          <span>Entities ({entities.length})</span>
        </div>
        <div className="flex items-center gap-1.5">
          <button
            onClick={() => createEntity("New Entity", selectedEntityId)}
            className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-[#1C1E24] hover:bg-[#23262D] text-[#8B919C] hover:text-[#E8EAED] transition-colors border border-[#23262D] cursor-pointer"
            title="Create Entity"
          >
            <Plus className="w-3 h-3 text-[#3DD68C]" />
            <span className="text-[10px]">New</span>
          </button>
          <button
            onClick={() => fetchSceneTree()}
            className="p-1 rounded hover:bg-[#1C1E24] text-[#8B919C] hover:text-[#E8EAED] transition-colors cursor-pointer"
            title="Refresh Scene Tree"
          >
            <RefreshCw className={`w-3 h-3 ${isLoading ? "animate-spin" : ""}`} />
          </button>
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
