import React, { useState, useEffect } from "react";
import { Minus, Square, X, Copy, Command } from "lucide-react";
import { useCommandStore } from "../stores/commandStore";

interface TitleBarProps {
  projectName?: string;
}

export const TitleBar: React.FC<TitleBarProps> = ({ projectName = "templates/hello" }) => {
  const [isMaximized, setIsMaximized] = useState(false);
  const openCommandPalette = useCommandStore((s) => s.open);

  const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

  useEffect(() => {
    if (!isTauri) return;

    let unlisten: (() => void) | undefined;
    import("@tauri-apps/api/window")
      .then(({ getCurrentWindow }) => {
        const appWindow = getCurrentWindow();
        appWindow.isMaximized().then(setIsMaximized);
        appWindow
          .onResized(() => {
            appWindow.isMaximized().then(setIsMaximized);
          })
          .then((fn) => {
            unlisten = fn;
          });
      })
      .catch(() => {
        // Browser preview mode
      });

    return () => {
      unlisten?.();
    };
  }, [isTauri]);

  const handleMinimize = async () => {
    if (isTauri) {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      await getCurrentWindow().minimize();
    }
  };

  const handleMaximize = async () => {
    if (isTauri) {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      await getCurrentWindow().toggleMaximize();
      const max = await getCurrentWindow().isMaximized();
      setIsMaximized(max);
    } else {
      setIsMaximized(!isMaximized);
    }
  };

  const handleClose = async () => {
    if (isTauri) {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      await getCurrentWindow().close();
    }
  };

  return (
    <header
      data-tauri-drag-region
      className="h-10 w-full bg-[#0A0B0D] border-b border-[#23262D] flex items-center justify-between px-3 select-none flex-shrink-0 z-50 text-[13px] font-sans"
    >
      {/* Left: App Icon + Name */}
      <div className="flex items-center gap-2 pointer-events-none">
        {/* Stylized Forge Flame Emblem */}
        <div className="w-5 h-5 rounded-md bg-gradient-to-tr from-[#FF6A2B] to-[#7C5CFF] flex items-center justify-center shadow-sm shadow-[#FF6A2B]/20">
          <div className="w-2.5 h-2.5 bg-[#0A0B0D] rounded-sm transform rotate-45" />
        </div>
        <span className="font-semibold text-[#E8EAED] tracking-tight">Forge</span>
        <span className="text-[11px] px-1.5 py-0.5 rounded bg-[#181B20] text-[#8B919C] border border-[#23262D] font-mono">
          STUDIO
        </span>
      </div>

      {/* Center: Project Name & Quick Search */}
      <div data-tauri-drag-region className="flex items-center gap-2 pointer-events-auto">
        <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-[#121418] border border-[#23262D] text-[#8B919C] text-xs">
          <span className="text-[#E8EAED] font-medium">{projectName}</span>
          <span className="text-[#23262D]">/</span>
          <span className="text-[#8B919C]">scene.json</span>
        </div>

        <button
          onClick={openCommandPalette}
          className="flex items-center gap-1 px-2 py-1 rounded bg-[#121418] hover:bg-[#1C1E24] text-[#8B919C] hover:text-[#E8EAED] border border-[#23262D] transition-colors duration-150 text-xs cursor-pointer"
          title="Command Palette (Ctrl+K)"
        >
          <Command className="w-3 h-3 text-[#7C5CFF]" />
          <span>Ctrl+K</span>
        </button>
      </div>

      {/* Right: Window Controls */}
      <div className="flex items-center gap-0.5 pointer-events-auto -mr-1">
        <button
          onClick={handleMinimize}
          className="w-8 h-7 flex items-center justify-center rounded hover:bg-[#181B20] text-[#8B919C] hover:text-[#E8EAED] transition-colors duration-150 cursor-pointer"
          aria-label="Minimize"
        >
          <Minus className="w-3.5 h-3.5" />
        </button>

        <button
          onClick={handleMaximize}
          className="w-8 h-7 flex items-center justify-center rounded hover:bg-[#181B20] text-[#8B919C] hover:text-[#E8EAED] transition-colors duration-150 cursor-pointer"
          aria-label={isMaximized ? "Restore" : "Maximize"}
        >
          {isMaximized ? <Copy className="w-3 h-3" /> : <Square className="w-3 h-3" />}
        </button>

        <button
          onClick={handleClose}
          className="w-8 h-7 flex items-center justify-center rounded hover:bg-[#FF4D4F] text-[#8B919C] hover:text-white transition-colors duration-150 cursor-pointer"
          aria-label="Close"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>
    </header>
  );
};
