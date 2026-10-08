import React, { useRef, useEffect, useCallback } from "react";
import { Play, RotateCcw, Maximize2, Crosshair, AlertTriangle, Monitor, Radio } from "lucide-react";
import { useRuntimeStore, type ViewportBounds } from "../../stores/runtimeStore";

export const ViewportPanel: React.FC = () => {
  const containerRef = useRef<HTMLDivElement>(null);
  const streamAnimRef = useRef<number | null>(null);

  const {
    status,
    mode,
    port,
    fps,
    crashedReason,
    streamFrame,
    launchRuntime,
    restartRuntime,
    setMode,
    updateBounds,
    focusViewport,
    pollStreamFrame,
  } = useRuntimeStore();

  // Send physical/logical bounding rect to Rust backend whenever moved or resized
  const measureAndSendBounds = useCallback(() => {
    if (!containerRef.current) return;
    const rect = containerRef.current.getBoundingClientRect();
    const isVisible =
      document.visibilityState === "visible" &&
      rect.width > 20 &&
      rect.height > 20 &&
      rect.bottom > 0 &&
      rect.right > 0 &&
      rect.top < window.innerHeight &&
      rect.left < window.innerWidth;

    const bounds: ViewportBounds = {
      x: rect.left,
      y: rect.top,
      width: rect.width,
      height: rect.height,
      visible: isVisible && status === "running" && mode === "embed",
    };

    updateBounds(bounds);
  }, [status, mode, updateBounds]);

  // Set up ResizeObserver and window scroll/resize listeners
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    measureAndSendBounds();

    const resizeObserver = new ResizeObserver(() => {
      measureAndSendBounds();
    });
    resizeObserver.observe(el);

    const handleWindowChange = () => {
      measureAndSendBounds();
    };

    window.addEventListener("resize", handleWindowChange);
    window.addEventListener("scroll", handleWindowChange, true);
    document.addEventListener("visibilitychange", handleWindowChange);

    // Initial measurement check after DOM settling
    const timer = setTimeout(measureAndSendBounds, 100);

    return () => {
      clearTimeout(timer);
      resizeObserver.disconnect();
      window.removeEventListener("resize", handleWindowChange);
      window.removeEventListener("scroll", handleWindowChange, true);
      document.removeEventListener("visibilitychange", handleWindowChange);
      // When unmounting/tabbing away, hide child window
      updateBounds({ x: 0, y: 0, width: 0, height: 0, visible: false });
    };
  }, [measureAndSendBounds, updateBounds]);

  // Offscreen streaming loop when in stream fallback mode
  useEffect(() => {
    if (mode !== "stream" || status !== "running") {
      if (streamAnimRef.current) {
        cancelAnimationFrame(streamAnimRef.current);
        streamAnimRef.current = null;
      }
      return;
    }

    let isPolling = false;

    const poll = async () => {
      if (isPolling) return;
      isPolling = true;
      try {
        await pollStreamFrame();
      } finally {
        isPolling = false;
      }
    };

    const timerId = setInterval(poll, 33); // ~30 FPS poll

    return () => {
      clearInterval(timerId);
    };
  }, [mode, status, pollStreamFrame]);

  const handleLaunch = () => {
    launchRuntime();
  };

  const handleRestart = () => {
    restartRuntime();
  };

  const toggleMode = () => {
    const nextMode = mode === "embed" ? "stream" : "embed";
    setMode(nextMode);
  };

  return (
    <div className="h-full w-full bg-[#0A0B0D] flex flex-col select-none relative overflow-hidden">
      {/* Top Overlay Controls Bar */}
      <div className="h-9 px-3 bg-[#121418]/90 backdrop-blur-xs border-b border-[#23262D] flex items-center justify-between text-xs z-20">
        <div className="flex items-center gap-1.5">
          {status !== "running" ? (
            <button
              onClick={handleLaunch}
              className="flex items-center gap-1.5 px-2.5 py-1 rounded bg-[#FF6A2B] hover:bg-[#E05A20] text-white font-medium text-xs transition-colors duration-150 cursor-pointer shadow-sm shadow-[#FF6A2B]/20"
            >
              <Play className="w-3 h-3 fill-white" />
              <span>Launch Scene</span>
            </button>
          ) : (
            <button
              onClick={handleRestart}
              className="flex items-center gap-1.5 px-2.5 py-1 rounded bg-[#181B20] hover:bg-[#22262E] text-[#E8EAED] border border-[#23262D] font-medium text-xs transition-colors duration-150 cursor-pointer"
              title="Restart Runtime"
            >
              <RotateCcw className="w-3 h-3 text-[#FF6A2B]" />
              <span>Restart</span>
            </button>
          )}

          {/* Render Mode Switcher */}
          <button
            onClick={toggleMode}
            className={`flex items-center gap-1 px-2 py-1 rounded border text-[11px] font-mono transition-colors duration-150 cursor-pointer ${
              mode === "embed"
                ? "bg-[#181B20] border-[#23262D] text-[#8B919C] hover:text-[#E8EAED]"
                : "bg-[#7C5CFF]/15 border-[#7C5CFF]/40 text-[#7C5CFF] hover:bg-[#7C5CFF]/25"
            }`}
            title="Toggle between Native Win32 HWND Embed and Streamed Frame Buffer fallback"
          >
            {mode === "embed" ? (
              <>
                <Monitor className="w-3 h-3 text-[#3DD68C]" />
                <span>HWND Embed</span>
              </>
            ) : (
              <>
                <Radio className="w-3 h-3 text-[#7C5CFF] animate-pulse" />
                <span>Stream Fallback</span>
              </>
            )}
          </button>
        </div>

        <div className="flex items-center gap-3 text-[#8B919C] text-[11px] font-mono">
          {status === "running" && (
            <span className="flex items-center gap-1.5">
              <span className="w-2 h-2 rounded-full bg-[#3DD68C] animate-pulse" />
              <span className="text-[#E8EAED]">{fps.toFixed(1)} FPS</span>
            </span>
          )}
          {port && <span>Port: {port}</span>}
          <button
            onClick={measureAndSendBounds}
            className="p-1 rounded hover:bg-[#1C1E24] hover:text-[#E8EAED] transition-colors cursor-pointer"
            title="Re-align Viewport"
          >
            <Maximize2 className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Main Viewport Content Area */}
      <div
        ref={containerRef}
        onClick={focusViewport}
        className="flex-1 w-full h-full relative flex items-center justify-center bg-[#0A0B0D]"
      >
        {/* State: Crashed */}
        {status === "crashed" && (
          <div className="absolute inset-0 z-30 flex flex-col items-center justify-center bg-[#0A0B0D]/90 backdrop-blur-sm p-6 text-center animate-in fade-in duration-200">
            <div className="p-3 rounded-full bg-[#FF4D4F]/10 border border-[#FF4D4F]/30 mb-3 text-[#FF4D4F]">
              <AlertTriangle className="w-8 h-8" />
            </div>
            <h3 className="text-sm font-semibold text-[#E8EAED] mb-1">
              Engine Runtime Process Crashed
            </h3>
            <p className="text-xs text-[#8B919C] max-w-md font-mono mb-4">
              {crashedReason ||
                "The child engine process exited unexpectedly. Studio remains fully operational."}
            </p>
            <button
              onClick={handleRestart}
              className="flex items-center gap-2 px-4 py-2 rounded-lg bg-[#FF6A2B] hover:bg-[#E05A20] text-white text-xs font-semibold shadow-lg shadow-[#FF6A2B]/25 transition-all cursor-pointer"
            >
              <RotateCcw className="w-3.5 h-3.5" />
              <span>Restart Engine</span>
            </button>
          </div>
        )}

        {/* State: Stopped / Starting */}
        {status === "stopped" && (
          <div className="flex flex-col items-center justify-center p-6 text-center z-10">
            <Crosshair className="w-10 h-10 text-[#23262D] mb-3" />
            <span className="text-sm font-semibold text-[#8B919C] mb-1">Engine Viewport Ready</span>
            <span className="text-xs text-[#8B919C]/70 mb-4 max-w-xs">
              Click below or press Ctrl+K to launch the hello scene inside the viewport panel.
            </span>
            <button
              onClick={handleLaunch}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-[#FF6A2B] hover:bg-[#E05A20] text-white font-medium text-xs transition-colors cursor-pointer"
            >
              <Play className="w-3.5 h-3.5 fill-white" />
              <span>Launch hello scene</span>
            </button>
          </div>
        )}

        {status === "starting" && (
          <div className="flex flex-col items-center justify-center p-6 text-center z-10">
            <div className="w-6 h-6 border-2 border-[#7C5CFF] border-t-transparent rounded-full animate-spin mb-3" />
            <span className="text-xs text-[#8B919C] font-mono">
              Initializing WGPU engine runtime...
            </span>
          </div>
        )}

        {/* State: Stream Mode Active */}
        {status === "running" && mode === "stream" && (
          <div className="w-full h-full flex items-center justify-center bg-black overflow-hidden relative">
            {streamFrame ? (
              <img
                src={`data:image/png;base64,${streamFrame}`}
                alt="Streamed Engine Viewport Frame"
                className="w-full h-full object-contain pointer-events-none select-none"
              />
            ) : (
              <div className="flex flex-col items-center justify-center text-xs text-[#8B919C] font-mono">
                <Radio className="w-5 h-5 text-[#7C5CFF] animate-pulse mb-2" />
                <span>Streaming frames from headless offscreen buffer...</span>
              </div>
            )}
            <div className="absolute top-2 right-2 px-2 py-0.5 rounded bg-black/60 border border-[#23262D] text-[10px] text-[#7C5CFF] font-mono">
              Stream Buffer
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
