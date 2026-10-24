import React, { useState, useEffect, useRef } from "react";
import {
  Sliders,
  Plus,
  Trash2,
  ChevronDown,
  ChevronRight,
  Move,
  Hash,
  Camera,
  Shield,
  Cpu,
  Volume2,
  FileCode,
  Grid,
} from "lucide-react";
import { useSceneStore } from "../../stores/sceneStore";
import { useEditStore } from "../../stores/editStore";
import { useRuntimeStore } from "../../stores/runtimeStore";
import type { ComponentFieldSchema } from "@forge/protocol";

// Drag-to-scrub number input component
export const ScrubbableNumberInput: React.FC<{
  value: number;
  step?: number;
  min?: number;
  max?: number;
  onChange: (val: number) => void;
  onCommit: (val: number) => void;
  label?: string;
}> = ({ value, step = 0.1, min, max, onChange, onCommit, label }) => {
  const [internalVal, setInternalVal] = useState(value);
  const isDragging = useRef(false);
  const startX = useRef(0);
  const startVal = useRef(value);

  useEffect(() => {
    setInternalVal(value);
  }, [value]);

  const handleMouseDown = (e: React.MouseEvent) => {
    isDragging.current = true;
    startX.current = e.clientX;
    startVal.current = internalVal;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      if (!isDragging.current) return;
      const delta = moveEvent.clientX - startX.current;
      let next = startVal.current + delta * step;
      if (min !== undefined) next = Math.max(min, next);
      if (max !== undefined) next = Math.min(max, next);
      const rounded = Math.round(next * 100) / 100;
      setInternalVal(rounded);
      onChange(rounded);
    };

    const handleMouseUp = () => {
      if (isDragging.current) {
        isDragging.current = false;
        onCommit(internalVal);
      }
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
  };

  const handleBlur = () => {
    onCommit(internalVal);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      onCommit(internalVal);
      (e.target as HTMLInputElement).blur();
    }
  };

  return (
    <div className="flex items-center gap-1 bg-[#121418] border border-[#23262D] rounded px-1.5 py-0.5 focus-within:border-[#7C5CFF] group">
      {label && (
        <span
          onMouseDown={handleMouseDown}
          className="text-[10px] text-[#8B919C] font-mono cursor-ew-resize select-none group-hover:text-[#FF6A2B] font-bold"
          title="Drag left/right to scrub"
        >
          {label}
        </span>
      )}
      <input
        type="number"
        value={Number.isNaN(internalVal) ? 0 : internalVal}
        step={step}
        min={min}
        max={max}
        onChange={(e) => {
          const val = parseFloat(e.target.value) || 0;
          setInternalVal(val);
          onChange(val);
        }}
        onBlur={handleBlur}
        onKeyDown={handleKeyDown}
        className="w-full bg-transparent text-right font-mono text-[11px] text-[#E8EAED] outline-none [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
      />
    </div>
  );
};

export const InspectorPanel: React.FC = () => {
  const { selectedEntity, fetchSceneTree } = useSceneStore();
  const { componentSchemas, fetchComponentSchemas } = useEditStore();
  const { bridgeClient } = useRuntimeStore();

  const [expandedComponents, setExpandedComponents] = useState<Record<string, boolean>>({});
  const [componentValues, setComponentValues] = useState<Record<string, Record<string, unknown>>>(
    {},
  );
  const [showAddMenu, setShowAddMenu] = useState(false);
  const [availableAssets, setAvailableAssets] = useState<
    Array<{ id: string; name: string; kind: string }>
  >([]);

  useEffect(() => {
    fetchComponentSchemas();
  }, [fetchComponentSchemas]);

  // Fetch asset list for asset pickers
  useEffect(() => {
    if (bridgeClient && bridgeClient.isConnected()) {
      bridgeClient
        .call<{ assets: Array<{ id: string; name: string; kind: string }> }>("asset.list", {})
        .then((res) => {
          if (res?.assets) setAvailableAssets(res.assets);
        })
        .catch(() => {});
    }
  }, [bridgeClient]);

  // When selected entity changes, expand components by default
  useEffect(() => {
    if (selectedEntity) {
      const exp: Record<string, boolean> = {};
      for (const comp of selectedEntity.components) {
        exp[comp] = true;
      }
      setExpandedComponents(exp);
      // Fetch or initialize entity component values
      setComponentValues({});
    }
  }, [selectedEntity]);

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

  const toggleComponent = (comp: string) => {
    setExpandedComponents((prev) => ({ ...prev, [comp]: !prev[comp] }));
  };

  const handleFieldChange = (compName: string, fieldName: string, val: unknown) => {
    setComponentValues((prev) => ({
      ...prev,
      [compName]: {
        ...(prev[compName] || {}),
        [fieldName]: val,
      },
    }));
  };

  const commitFieldChange = async (compName: string, fieldName: string, val: unknown) => {
    if (!bridgeClient || !bridgeClient.isConnected() || !selectedEntity) return;

    try {
      await bridgeClient.setComponent(selectedEntity.id, compName, {
        [fieldName]: typeof val === "object" ? JSON.stringify(val) : String(val),
      });
      await fetchSceneTree();
    } catch (err) {
      console.error("Failed to commit component field:", err);
    }
  };

  const handleAddComponent = async (compName: string) => {
    setShowAddMenu(false);
    if (!bridgeClient || !bridgeClient.isConnected() || !selectedEntity) return;

    const schema = componentSchemas[compName];
    const defaultData: Record<string, string> = {};
    if (schema?.fields) {
      for (const f of schema.fields) {
        if (f.default_value !== undefined && f.default_value !== null) {
          defaultData[f.name] =
            typeof f.default_value === "object"
              ? JSON.stringify(f.default_value)
              : String(f.default_value);
        }
      }
    }

    try {
      await bridgeClient.setComponent(selectedEntity.id, compName, defaultData);
      await fetchSceneTree();
    } catch (err) {
      console.error("Failed to add component:", err);
    }
  };

  const handleRemoveComponent = async (compName: string) => {
    if (!bridgeClient || !bridgeClient.isConnected() || !selectedEntity) return;

    try {
      await bridgeClient.removeComponent(selectedEntity.id, compName);
      await fetchSceneTree();
    } catch (err) {
      console.error("Failed to remove component:", err);
    }
  };

  const getComponentIcon = (name: string) => {
    const lower = name.toLowerCase();
    if (lower.includes("transform")) return Move;
    if (lower.includes("sprite")) return Hash;
    if (lower.includes("camera")) return Camera;
    if (lower.includes("collider") || lower.includes("body")) return Shield;
    if (lower.includes("audio")) return Volume2;
    if (lower.includes("script")) return FileCode;
    if (lower.includes("tilemap")) return Grid;
    return Cpu;
  };

  const renderFieldWidget = (compName: string, field: ComponentFieldSchema) => {
    const hint = field.hint;
    const widgetType = hint?.widget || field.field_type;
    const currentVal = componentValues[compName]?.[field.name] ?? field.default_value;

    switch (widgetType) {
      case "number": {
        const num =
          typeof currentVal === "number" ? currentVal : parseFloat(String(currentVal)) || 0;
        return (
          <ScrubbableNumberInput
            value={num}
            step={hint?.step ?? 0.1}
            min={hint?.min ?? undefined}
            max={hint?.max ?? undefined}
            onChange={(v) => handleFieldChange(compName, field.name, v)}
            onCommit={(v) => commitFieldChange(compName, field.name, v)}
          />
        );
      }

      case "vector2": {
        const vec = Array.isArray(currentVal) ? (currentVal as [number, number]) : [0, 0];
        return (
          <div className="grid grid-cols-2 gap-1.5">
            <ScrubbableNumberInput
              label="X"
              value={vec[0] ?? 0}
              step={hint?.step ?? 0.5}
              min={hint?.min ?? undefined}
              max={hint?.max ?? undefined}
              onChange={(x) => handleFieldChange(compName, field.name, [x, vec[1] ?? 0])}
              onCommit={(x) => commitFieldChange(compName, field.name, [x, vec[1] ?? 0])}
            />
            <ScrubbableNumberInput
              label="Y"
              value={vec[1] ?? 0}
              step={hint?.step ?? 0.5}
              min={hint?.min ?? undefined}
              max={hint?.max ?? undefined}
              onChange={(y) => handleFieldChange(compName, field.name, [vec[0] ?? 0, y])}
              onCommit={(y) => commitFieldChange(compName, field.name, [vec[0] ?? 0, y])}
            />
          </div>
        );
      }

      case "bool": {
        const checked = Boolean(currentVal);
        return (
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={checked}
              onChange={(e) => {
                const next = e.target.checked;
                handleFieldChange(compName, field.name, next);
                commitFieldChange(compName, field.name, next);
              }}
              className="rounded bg-[#121418] border-[#23262D] text-[#7C5CFF] focus:ring-0 focus:ring-offset-0 cursor-pointer"
            />
            <span className="text-[#8B919C] text-[11px] font-mono">
              {checked ? "true" : "false"}
            </span>
          </label>
        );
      }

      case "enum": {
        const options = hint?.options || [];
        const strVal = String(currentVal ?? "");
        return (
          <select
            value={strVal}
            onChange={(e) => {
              const next = e.target.value;
              handleFieldChange(compName, field.name, next);
              commitFieldChange(compName, field.name, next);
            }}
            className="w-full px-2 py-1 bg-[#121418] border border-[#23262D] rounded text-[#E8EAED] text-[11px] outline-none font-sans"
          >
            {options.map((opt) => (
              <option key={opt} value={opt}>
                {opt}
              </option>
            ))}
          </select>
        );
      }

      case "color": {
        const colorArr = Array.isArray(currentVal) ? (currentVal as number[]) : [1, 1, 1, 1];
        const hex = `#${Math.round((colorArr[0] ?? 1) * 255)
          .toString(16)
          .padStart(2, "0")}${Math.round((colorArr[1] ?? 1) * 255)
          .toString(16)
          .padStart(2, "0")}${Math.round((colorArr[2] ?? 1) * 255)
          .toString(16)
          .padStart(2, "0")}`;
        return (
          <div className="flex items-center gap-2">
            <input
              type="color"
              value={hex}
              onChange={(e) => {
                const hexVal = e.target.value;
                const r = parseInt(hexVal.slice(1, 3), 16) / 255;
                const g = parseInt(hexVal.slice(3, 5), 16) / 255;
                const b = parseInt(hexVal.slice(5, 7), 16) / 255;
                const next = [r, g, b, colorArr[3] ?? 1.0];
                handleFieldChange(compName, field.name, next);
                commitFieldChange(compName, field.name, next);
              }}
              className="w-6 h-6 rounded border border-[#23262D] bg-transparent cursor-pointer p-0"
            />
            <span className="font-mono text-[10px] text-[#8B919C]">{hex.toUpperCase()}</span>
          </div>
        );
      }

      case "asset": {
        const currentAssetId = String(currentVal ?? "");
        return (
          <div className="space-y-1">
            <select
              value={currentAssetId}
              onChange={(e) => {
                const next = e.target.value;
                handleFieldChange(compName, field.name, next);
                commitFieldChange(compName, field.name, next);
              }}
              className="w-full px-2 py-1 bg-[#121418] border border-[#23262D] rounded text-[#E8EAED] text-[11px] outline-none font-sans"
            >
              <option value="">(None)</option>
              {availableAssets.map((ast) => (
                <option key={ast.id} value={ast.id}>
                  {ast.name} ({ast.kind})
                </option>
              ))}
            </select>
          </div>
        );
      }

      default: {
        const textVal = String(currentVal ?? "");
        return (
          <input
            type="text"
            value={textVal}
            onChange={(e) => handleFieldChange(compName, field.name, e.target.value)}
            onBlur={() =>
              commitFieldChange(
                compName,
                field.name,
                componentValues[compName]?.[field.name] ?? textVal,
              )
            }
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                commitFieldChange(
                  compName,
                  field.name,
                  componentValues[compName]?.[field.name] ?? textVal,
                );
                (e.target as HTMLInputElement).blur();
              }
            }}
            className="w-full px-2 py-1 bg-[#121418] border border-[#23262D] rounded text-[#E8EAED] font-medium outline-none text-[11px]"
          />
        );
      }
    }
  };

  const missingComponents = Object.keys(componentSchemas).filter(
    (name) => !selectedEntity.components.includes(name),
  );

  return (
    <div className="h-full w-full bg-[#121418] text-[#E8EAED] flex flex-col select-none text-xs">
      {/* Header */}
      <div className="p-2 border-b border-[#23262D] flex items-center justify-between bg-[#0A0B0D]/50">
        <div className="flex items-center gap-1.5 font-medium text-[#8B919C]">
          <Sliders className="w-3.5 h-3.5 text-[#FF6A2B]" />
          <span>Inspector</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-[10px] text-[#3DD68C] font-mono truncate max-w-[120px]">
            {selectedEntity.id}
          </span>
          <div className="relative">
            <button
              onClick={() => setShowAddMenu(!showAddMenu)}
              className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-[#1C1E24] hover:bg-[#23262D] text-[#8B919C] hover:text-[#E8EAED] transition-colors border border-[#23262D] cursor-pointer"
              title="Add Component"
            >
              <Plus className="w-3 h-3 text-[#3DD68C]" />
              <span className="text-[10px]">Add</span>
            </button>
            {showAddMenu && (
              <div className="absolute right-0 top-full mt-1 w-44 bg-[#181B20] border border-[#23262D] rounded-lg shadow-xl z-50 py-1 max-h-56 overflow-y-auto">
                {missingComponents.length === 0 ? (
                  <div className="px-3 py-2 text-[11px] text-[#8B919C]">All components added</div>
                ) : (
                  missingComponents.map((name) => (
                    <button
                      key={name}
                      onClick={() => handleAddComponent(name)}
                      className="w-full text-left px-3 py-1.5 hover:bg-[#23262D] text-[#E8EAED] text-[11px] flex items-center gap-2 cursor-pointer"
                    >
                      <Plus className="w-3 h-3 text-[#7C5CFF]" />
                      <span>{componentSchemas[name]?.display_name || name}</span>
                    </button>
                  ))
                )}
              </div>
            )}
          </div>
        </div>
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
              defaultValue={selectedEntity.name}
              onBlur={(e) => {
                if (e.target.value !== selectedEntity.name) {
                  useSceneStore.getState().renameEntity(selectedEntity.id, e.target.value);
                }
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  (e.target as HTMLInputElement).blur();
                }
              }}
              className="flex-1 px-2 py-1 bg-[#181B20] border border-[#23262D] rounded text-[#E8EAED] font-medium outline-none text-[11px]"
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

        {/* Dynamic Components List Driven by Schemas */}
        <div className="space-y-2.5">
          <label className="text-[11px] font-semibold text-[#8B919C] uppercase tracking-wider">
            Components ({selectedEntity.components.length})
          </label>

          {selectedEntity.components.map((compName) => {
            const Icon = getComponentIcon(compName);
            const schema = componentSchemas[compName];
            const isExpanded = expandedComponents[compName] ?? true;

            return (
              <div
                key={compName}
                className="rounded-lg border border-[#23262D] bg-[#181B20]/60 overflow-hidden"
              >
                {/* Header */}
                <div
                  onClick={() => toggleComponent(compName)}
                  className="px-2.5 py-1.5 flex items-center justify-between bg-[#181B20] cursor-pointer hover:bg-[#1E2228] transition-colors"
                >
                  <div className="flex items-center gap-1.5 font-medium text-[#E8EAED]">
                    {isExpanded ? (
                      <ChevronDown className="w-3.5 h-3.5 text-[#8B919C]" />
                    ) : (
                      <ChevronRight className="w-3.5 h-3.5 text-[#8B919C]" />
                    )}
                    <Icon className="w-3.5 h-3.5 text-[#7C5CFF]" />
                    <span>{schema?.display_name || compName}</span>
                  </div>
                  <div className="flex items-center gap-1">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleRemoveComponent(compName);
                      }}
                      className="p-1 rounded hover:bg-[#2A2E37] text-[#8B919C] hover:text-[#FF4D4F] transition-colors cursor-pointer"
                      title="Remove Component"
                    >
                      <Trash2 className="w-3 h-3" />
                    </button>
                  </div>
                </div>

                {/* Body (Dynamic Fields) */}
                {isExpanded && (
                  <div className="p-2.5 space-y-2 border-t border-[#23262D]/60 bg-[#121418]/30">
                    {schema?.fields && schema.fields.length > 0 ? (
                      schema.fields.map((field) => (
                        <div key={field.name} className="flex flex-col gap-1">
                          <div className="flex items-center justify-between">
                            <span className="text-[11px] text-[#8B919C] capitalize">
                              {field.name.replace(/_/g, " ")}
                            </span>
                            {field.hint?.widget && (
                              <span className="text-[9px] text-[#8B919C]/60 font-mono">
                                {field.hint.widget}
                              </span>
                            )}
                          </div>
                          {renderFieldWidget(compName, field)}
                        </div>
                      ))
                    ) : (
                      <div className="text-[11px] text-[#8B919C]/80 italic py-1">
                        Active ECS Component
                      </div>
                    )}
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
