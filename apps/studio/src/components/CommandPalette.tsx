import React, { useEffect } from "react";
import { Command } from "cmdk";
import {
  RotateCcw,
  Settings,
  Layers,
  Monitor,
  Code2,
  Sliders,
  Bot,
  Terminal,
  Sparkles,
  Camera,
  Activity,
  Play,
  Square,
} from "lucide-react";
import { useCommandStore } from "../stores/commandStore";
import { useLayoutStore } from "../stores/layoutStore";
import { useRuntimeStore } from "../stores/runtimeStore";

interface CommandPaletteProps {
  onTogglePanel?: (panelId: string) => void;
  onResetLayout?: () => void;
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({ onTogglePanel, onResetLayout }) => {
  const { isOpen, close, toggle, search, setSearch } = useCommandStore();
  const resetStoreLayout = useLayoutStore((s) => s.resetLayout);
  const { launchRuntime, restartRuntime, stopRuntime } = useRuntimeStore();

  // Keyboard shortcut listener for Ctrl+K / Cmd+K
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        toggle();
      }
      if (e.key === "Escape" && isOpen) {
        e.preventDefault();
        close();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, toggle, close]);

  if (!isOpen) return null;

  const handleAction = (action: () => void) => {
    action();
    close();
  };

  const handleReset = () => {
    resetStoreLayout();
    onResetLayout?.();
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-24 bg-black/60 backdrop-blur-xs animate-in fade-in-0 duration-150"
      onClick={close}
    >
      <div onClick={(e) => e.stopPropagation()} className="w-full max-w-[620px] px-4">
        <Command
          value={search}
          onValueChange={setSearch}
          className="border border-[#23262D] bg-[#121418] shadow-2xl rounded-xl overflow-hidden font-sans"
        >
          <div className="flex items-center px-3.5 border-b border-[#23262D] gap-2">
            <Sparkles className="w-4 h-4 text-[#7C5CFF]" />
            <Command.Input
              autoFocus
              placeholder="Type a command or search..."
              className="w-full py-3.5 bg-transparent text-[#E8EAED] placeholder-[#8B919C] text-sm outline-none"
            />
            <kbd className="px-1.5 py-0.5 text-[10px] rounded bg-[#181B20] text-[#8B919C] border border-[#23262D] font-mono">
              ESC
            </kbd>
          </div>

          <Command.List className="max-h-[340px] overflow-y-auto p-2 text-sm text-[#E8EAED]">
            <Command.Empty className="py-6 text-center text-xs text-[#8B919C]">
              No matching commands found.
            </Command.Empty>

            <Command.Group
              heading="Engine & Runtime"
              className="text-xs text-[#8B919C] px-2 py-1.5"
            >
              <Command.Item
                onSelect={() => handleAction(() => launchRuntime("templates/hello/scene.json"))}
                className="flex items-center justify-between px-2.5 py-2 rounded-md hover:bg-[#1C1E24] data-[selected=true]:bg-[#20232B] data-[selected=true]:text-white cursor-pointer transition-colors duration-100"
              >
                <div className="flex items-center gap-2.5">
                  <Play className="w-4 h-4 text-[#FF6A2B] fill-[#FF6A2B]" />
                  <span className="font-medium text-white">Launch hello scene</span>
                </div>
                <span className="text-[11px] text-[#FF6A2B] font-mono">F5</span>
              </Command.Item>

              <Command.Item
                onSelect={() => handleAction(() => restartRuntime())}
                className="flex items-center justify-between px-2.5 py-2 rounded-md hover:bg-[#1C1E24] data-[selected=true]:bg-[#20232B] data-[selected=true]:text-white cursor-pointer transition-colors duration-100"
              >
                <div className="flex items-center gap-2.5">
                  <RotateCcw className="w-4 h-4 text-[#7C5CFF]" />
                  <span>Restart Engine</span>
                </div>
                <span className="text-[11px] text-[#8B919C] font-mono">Ctrl+Shift+F5</span>
              </Command.Item>

              <Command.Item
                onSelect={() => handleAction(() => stopRuntime())}
                className="flex items-center justify-between px-2.5 py-2 rounded-md hover:bg-[#1C1E24] data-[selected=true]:bg-[#20232B] data-[selected=true]:text-white cursor-pointer transition-colors duration-100"
              >
                <div className="flex items-center gap-2.5">
                  <Square className="w-4 h-4 text-[#FF4D4F]" />
                  <span>Stop Engine</span>
                </div>
                <span className="text-[11px] text-[#8B919C] font-mono">Shift+F5</span>
              </Command.Item>
            </Command.Group>

            <Command.Group
              heading="Workspace & Layout"
              className="text-xs text-[#8B919C] px-2 py-1.5 mt-1"
            >
              <Command.Item
                onSelect={() => handleAction(handleReset)}
                className="flex items-center justify-between px-2.5 py-2 rounded-md hover:bg-[#1C1E24] data-[selected=true]:bg-[#20232B] data-[selected=true]:text-white cursor-pointer transition-colors duration-100"
              >
                <div className="flex items-center gap-2.5">
                  <RotateCcw className="w-4 h-4 text-[#FF6A2B]" />
                  <span>Reset Dock Layout</span>
                </div>
                <span className="text-[11px] text-[#8B919C] font-mono">Alt+R</span>
              </Command.Item>

              <Command.Item
                onSelect={() =>
                  handleAction(() => alert("Studio settings will be available in next milestone."))
                }
                className="flex items-center justify-between px-2.5 py-2 rounded-md hover:bg-[#1C1E24] data-[selected=true]:bg-[#20232B] data-[selected=true]:text-white cursor-pointer transition-colors duration-100"
              >
                <div className="flex items-center gap-2.5">
                  <Settings className="w-4 h-4 text-[#8B919C]" />
                  <span>Open Settings</span>
                </div>
                <span className="text-[11px] text-[#8B919C] font-mono">Ctrl+,</span>
              </Command.Item>
            </Command.Group>

            <Command.Group
              heading="Toggle Panels"
              className="text-xs text-[#8B919C] px-2 py-1.5 mt-1"
            >
              <Command.Item
                onSelect={() => handleAction(() => onTogglePanel?.("scene_tree"))}
                className="flex items-center justify-between px-2.5 py-2 rounded-md hover:bg-[#1C1E24] data-[selected=true]:bg-[#20232B] data-[selected=true]:text-white cursor-pointer transition-colors duration-100"
              >
                <div className="flex items-center gap-2.5">
                  <Layers className="w-4 h-4 text-[#8B919C]" />
                  <span>Toggle Scene Tree</span>
                </div>
                <span className="text-[11px] text-[#8B919C] font-mono">Panel</span>
              </Command.Item>

              <Command.Item
                onSelect={() => handleAction(() => onTogglePanel?.("viewport"))}
                className="flex items-center justify-between px-2.5 py-2 rounded-md hover:bg-[#1C1E24] data-[selected=true]:bg-[#20232B] data-[selected=true]:text-white cursor-pointer transition-colors duration-100"
              >
                <div className="flex items-center gap-2.5">
                  <Monitor className="w-4 h-4 text-[#8B919C]" />
                  <span>Toggle Viewport</span>
                </div>
                <span className="text-[11px] text-[#8B919C] font-mono">Panel</span>
              </Command.Item>

              <Command.Item
                onSelect={() => handleAction(() => onTogglePanel?.("code_editor"))}
                className="flex items-center justify-between px-2.5 py-2 rounded-md hover:bg-[#1C1E24] data-[selected=true]:bg-[#20232B] data-[selected=true]:text-white cursor-pointer transition-colors duration-100"
              >
                <div className="flex items-center gap-2.5">
                  <Code2 className="w-4 h-4 text-[#8B919C]" />
                  <span>Toggle Code Editor</span>
                </div>
                <span className="text-[11px] text-[#8B919C] font-mono">Panel</span>
              </Command.Item>

              <Command.Item
                onSelect={() => handleAction(() => onTogglePanel?.("inspector"))}
                className="flex items-center justify-between px-2.5 py-2 rounded-md hover:bg-[#1C1E24] data-[selected=true]:bg-[#20232B] data-[selected=true]:text-white cursor-pointer transition-colors duration-100"
              >
                <div className="flex items-center gap-2.5">
                  <Sliders className="w-4 h-4 text-[#8B919C]" />
                  <span>Toggle Inspector</span>
                </div>
                <span className="text-[11px] text-[#8B919C] font-mono">Panel</span>
              </Command.Item>

              <Command.Item
                onSelect={() => handleAction(() => onTogglePanel?.("agent_chat"))}
                className="flex items-center justify-between px-2.5 py-2 rounded-md hover:bg-[#1C1E24] data-[selected=true]:bg-[#20232B] data-[selected=true]:text-white cursor-pointer transition-colors duration-100"
              >
                <div className="flex items-center gap-2.5">
                  <Bot className="w-4 h-4 text-[#7C5CFF]" />
                  <span>Toggle Agent Chat</span>
                </div>
                <span className="text-[11px] text-[#8B919C] font-mono">AI</span>
              </Command.Item>

              <Command.Item
                onSelect={() => handleAction(() => onTogglePanel?.("terminal"))}
                className="flex items-center justify-between px-2.5 py-2 rounded-md hover:bg-[#1C1E24] data-[selected=true]:bg-[#20232B] data-[selected=true]:text-white cursor-pointer transition-colors duration-100"
              >
                <div className="flex items-center gap-2.5">
                  <Terminal className="w-4 h-4 text-[#8B919C]" />
                  <span>Toggle Terminal</span>
                </div>
                <span className="text-[11px] text-[#8B919C] font-mono">Bottom</span>
              </Command.Item>
            </Command.Group>

            <Command.Group
              heading="Engine Tools (Bridge)"
              className="text-xs text-[#8B919C] px-2 py-1.5 mt-1"
            >
              <Command.Item
                onSelect={() => handleAction(() => alert("Triggering engine.ping over bridge..."))}
                className="flex items-center justify-between px-2.5 py-2 rounded-md hover:bg-[#1C1E24] data-[selected=true]:bg-[#20232B] data-[selected=true]:text-white cursor-pointer transition-colors duration-100"
              >
                <div className="flex items-center gap-2.5">
                  <Activity className="w-4 h-4 text-[#3DD68C]" />
                  <span>Engine: Ping Runtime Bridge</span>
                </div>
                <span className="text-[11px] text-[#3DD68C] font-mono">ping</span>
              </Command.Item>

              <Command.Item
                onSelect={() => handleAction(() => alert("Capturing game screenshot..."))}
                className="flex items-center justify-between px-2.5 py-2 rounded-md hover:bg-[#1C1E24] data-[selected=true]:bg-[#20232B] data-[selected=true]:text-white cursor-pointer transition-colors duration-100"
              >
                <div className="flex items-center gap-2.5">
                  <Camera className="w-4 h-4 text-[#F5A524]" />
                  <span>Engine: Capture Viewport Screenshot</span>
                </div>
                <span className="text-[11px] text-[#8B919C] font-mono">F12</span>
              </Command.Item>
            </Command.Group>
          </Command.List>

          <div className="px-3 py-2 bg-[#0A0B0D] border-t border-[#23262D] flex items-center justify-between text-[11px] text-[#8B919C]">
            <span>Navigate with ↑↓, select with Enter</span>
            <span className="font-mono text-[#7C5CFF]">Forge Studio v0.1.0</span>
          </div>
        </Command>
      </div>
    </div>
  );
};
