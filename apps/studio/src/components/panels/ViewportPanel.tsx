import React, { useRef, useEffect, useCallback } from "react";
import {
  Play,
  Pause,
  StepForward,
  Square,
  RotateCcw,
  Maximize2,
  Crosshair,
  AlertTriangle,
  Monitor,
  Radio,
  Move,
  RotateCw,
  Scaling,
  Grid,
  Magnet,
} from "lucide-react";
import { useRuntimeStore, type ViewportBounds } from "../../stores/runtimeStore";
import { usePlayStore } from "../../stores/playStore";

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
    bridgeClient,
    launchRuntime,
    restartRuntime,
    setMode,
    updateBounds,
    focusViewport,
    pollStreamFrame,
  } = useRuntimeStore();

  const {
    playState,
    activeGizmo,
    gridSnapEnabled,
    angleSnapEnabled,
    showGrid,
    play,
    pause,
    step,
    stop,
    setActiveGizmo,
    toggleGridSnap,
    toggleAngleSnap,
    toggleShowGrid,
  } = usePlayStore();

  // Global Keyboard shortcuts for Play/Pause/Step/Stop and Gizmos
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Don't intercept if typing in an input/textarea/select
      const activeEl = document.activeElement;
      if (
        activeEl &&
        (activeEl.tagName === "INPUT" ||
          activeEl.tagName === "TEXTAREA" ||
          activeEl.tagName === "SELECT" ||
          activeEl.isContentEditable)
      ) {
        return;
      }

      if (e.key === "F5") {
        e.preventDefault();
        if (e.shiftKey) {
          stop();
        } else if (playState === "playing") {
          pause();
        } else {
          play();
        }
      } else if (e.key === "F10") {
        e.preventDefault();
        step();
      } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "p") {
        e.preventDefault();
        if (playState === "playing") {
          pause();
        } else {
          play();
        }
      } else if (playState === "edit") {
        if (e.key.toLowerCase() === "w" && !e.ctrlKey && !e.metaKey && !e.altKey) {
          setActiveGizmo("translate");
        } else if (e.key.toLowerCase() === "e" && !e.ctrlKey && !e.metaKey && !e.altKey) {
          setActiveGizmo("rotate");
        } else if (e.key.toLowerCase() === "r" && !e.ctrlKey && !e.metaKey && !e.altKey) {
          setActiveGizmo("scale");
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [playState, play, pause, step, stop, setActiveGizmo]);

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

  // Streamed Input Event Forwarding with accurate Coordinate Mapping & DPI Handling
  const calculateFrameCoordinates = useCallback(
    (e: React.PointerEvent<HTMLDivElement>): [number, number] => {
      if (!containerRef.current) return [0, 0];
      const rect = containerRef.current.getBoundingClientRect();
      const cssX = Math.max(0, Math.min(e.clientX - rect.left, rect.width));
      const cssY = Math.max(0, Math.min(e.clientY - rect.top, rect.height));

      // In streamed mode, the headless engine renders to viewport frame bounds
      // We map CSS pixel offset proportionally to target engine frame resolution
      const targetWidth = rect.width;
      const targetHeight = rect.height;

      const normX = rect.width > 0 ? cssX / rect.width : 0;
      const normY = rect.height > 0 ? cssY / rect.height : 0;

      const frameX = normX * targetWidth;
      const frameY = normY * targetHeight;

      return [Math.round(frameX), Math.round(frameY)];
    },
    []
  );

  const forwardPointerEvent = useCallback(
    async (
      action: "mouse_down" | "mouse_up" | "mouse_move",
      e: React.PointerEvent<HTMLDivElement>
    ) => {
      if (mode !== "stream" || !bridgeClient || !bridgeClient.isConnected()) return;

      const [x, y] = calculateFrameCoordinates(e);
      let button = "left";
      if (e.button === 1) button = "middle";
      else if (e.button === 2) button = "right";

      try {
        await bridgeClient.simulateInput({
          action,
          button,
          position: [x, y],
          delta: [e.movementX || 0, e.movementY || 0],
        });
      } catch (err) {
        console.warn("Failed to forward stream pointer event:", err);
      }
    },
    [mode, bridgeClient, calculateFrameCoordinates]
  );

  const handlePointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    (e.target as HTMLElement).setPointerCapture?.(e.pointerId);
    forwardPointerEvent("mouse_down", e);
  };

  const handlePointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    forwardPointerEvent("mouse_move", e);
  };

  const handlePointerUp = (e: React.PointerEvent<HTMLDivElement>) => {
    try {
      (e.target as HTMLElement).releasePointerCapture?.(e.pointerId);
    } catch {
      // ignore
    }
    forwardPointerEvent("mouse_up", e);
  };

  const handleWheel = useCallback(
    async (e: React.WheelEvent<HTMLDivElement>) => {
      if (mode !== "stream" || !bridgeClient || !bridgeClient.isConnected()) return;
      if (!containerRef.current) return;

      const rect = containerRef.current.getBoundingClientRect();
      const cssX = Math.max(0, Math.min(e.clientX - rect.left, rect.width));
      const cssY = Math.max(0, Math.min(e.clientY - rect.top, rect.height));

      try {
        await bridgeClient.simulateInput({
          action: "mouse_wheel",
          position: [Math.round(cssX), Math.round(cssY)],
          delta: [e.deltaX, e.deltaY],
        });
      } catch (err) {
        console.warn("Failed to forward stream wheel event:", err);
      }
    },
    [mode, bridgeClient]
  );

  const handleKeyDownStream = useCallback(
    async (e: React.KeyboardEvent<HTMLDivElement>) => {
      if (mode !== "stream" || !bridgeClient || !bridgeClient.isConnected()) return;
      try {
        await bridgeClient.simulateInput({
          action: "key_down",
          key: e.key,
        });
      } catch (err) {
        console.warn("Failed to forward stream key event:", err);
      }
    },
    [mode, bridgeClient]
  );

  const handleKeyUpStream = useCallback(
    async (e: React.KeyboardEvent<HTMLDivElement>) => {
      if (mode !== "stream" || !bridgeClient || !bridgeClient.isConnected()) return;
      try {
        await bridgeClient.simulateInput({
          action: "key_up",
          key: e.key,
        });
      } catch (err) {
        console.warn("Failed to forward stream key event:", err);
      }
    },
    [mode, bridgeClient]
  );

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
            <div className="flex items-center gap-1">
              {/* Play / Pause / Step / Stop toolbar buttons */}
              {playState === "edit" ? (
                <button
                  onClick={play}
                  data-testid="play-btn"
                  className="flex items-center gap-1 px-2.5 py-1 rounded bg-[#3DD68C]/20 hover:bg-[#3DD68C]/30 text-[#3DD68C] border border-[#3DD68C]/40 font-medium text-xs transition-colors cursor-pointer"
                  title="Play (F5 / Ctrl+P)"
                >
                  <Play className="w-3 h-3 fill-[#3DD68C]" />
                  <span>Play</span>
                </button>
              ) : playState === "playing" ? (
                <button
                  onClick={pause}
                  data-testid="pause-btn"
                  className="flex items-center gap-1 px-2.5 py-1 rounded bg-[#FFB020]/20 hover:bg-[#FFB020]/30 text-[#FFB020] border border-[#FFB020]/40 font-medium text-xs transition-colors cursor-pointer"
                  title="Pause (F5 / Ctrl+P)"
                >
                  <Pause className="w-3 h-3 fill-[#FFB020]" />
                  <span>Pause</span>
                </button>
              ) : (
                <button
                  onClick={play}
                  data-testid="resume-btn"
                  className="flex items-center gap-1 px-2.5 py-1 rounded bg-[#3DD68C]/20 hover:bg-[#3DD68C]/30 text-[#3DD68C] border border-[#3DD68C]/40 font-medium text-xs transition-colors cursor-pointer"
                  title="Resume (F5 / Ctrl+P)"
                >
                  <Play className="w-3 h-3 fill-[#3DD68C]" />
                  <span>Resume</span>
                </button>
              )}

              {playState === "paused" && (
                <button
                  onClick={step}
                  data-testid="step-btn"
                  className="flex items-center gap-1 px-2 py-1 rounded bg-[#181B20] hover:bg-[#22262E] text-[#E8EAED] border border-[#23262D] font-medium text-xs transition-colors cursor-pointer"
                  title="Step 1 Frame (F10)"
                >
                  <StepForward className="w-3 h-3 text-[#3DD68C]" />
                  <span>Step</span>
                </button>
              )}

              {playState !== "edit" && (
                <button
                  onClick={stop}
                  data-testid="stop-btn"
                  className="flex items-center gap-1 px-2.5 py-1 rounded bg-[#FF4D4F]/20 hover:bg-[#FF4D4F]/30 text-[#FF4D4F] border border-[#FF4D4F]/40 font-medium text-xs transition-colors cursor-pointer"
                  title="Stop (Shift+F5)"
                >
                  <Square className="w-3 h-3 fill-[#FF4D4F]" />
                  <span>Stop</span>
                </button>
              )}

              <button
                onClick={handleRestart}
                className="flex items-center gap-1 p-1 rounded bg-[#181B20] hover:bg-[#22262E] text-[#8B919C] hover:text-[#E8EAED] border border-[#23262D] transition-colors cursor-pointer ml-1"
                title="Restart Runtime"
              >
                <RotateCcw className="w-3 h-3" />
              </button>
            </div>
          )}

          {/* Gizmo Tools Bar (W, E, R) */}
          <div className="h-4 w-px bg-[#23262D] mx-1" />
          <div className="flex items-center gap-0.5 bg-[#181B20] rounded border border-[#23262D] p-0.5">
            <button
              onClick={() => setActiveGizmo("translate")}
              data-testid="gizmo-translate-btn"
              className={`p-1 rounded transition-colors ${
                activeGizmo === "translate"
                  ? "bg-[#7C5CFF] text-white"
                  : "text-[#8B919C] hover:text-[#E8EAED]"
              }`}
              title="Move Tool (W)"
            >
              <Move className="w-3 h-3" />
            </button>
            <button
              onClick={() => setActiveGizmo("rotate")}
              data-testid="gizmo-rotate-btn"
              className={`p-1 rounded transition-colors ${
                activeGizmo === "rotate"
                  ? "bg-[#7C5CFF] text-white"
                  : "text-[#8B919C] hover:text-[#E8EAED]"
              }`}
              title="Rotate Tool (E)"
            >
              <RotateCw className="w-3 h-3" />
            </button>
            <button
              onClick={() => setActiveGizmo("scale")}
              data-testid="gizmo-scale-btn"
              className={`p-1 rounded transition-colors ${
                activeGizmo === "scale"
                  ? "bg-[#7C5CFF] text-white"
                  : "text-[#8B919C] hover:text-[#E8EAED]"
              }`}
              title="Scale Tool (R)"
            >
              <Scaling className="w-3 h-3" />
            </button>
          </div>

          {/* Snapping and Grid Toggles */}
          <div className="flex items-center gap-0.5 bg-[#181B20] rounded border border-[#23262D] p-0.5 ml-1">
            <button
              onClick={toggleShowGrid}
              className={`p-1 rounded transition-colors ${
                showGrid ? "text-[#3DD68C]" : "text-[#8B919C] hover:text-[#E8EAED]"
              }`}
              title="Toggle Grid (G)"
            >
              <Grid className="w-3 h-3" />
            </button>
            <button
              onClick={toggleGridSnap}
              className={`p-1 rounded transition-colors ${
                gridSnapEnabled ? "text-[#FF6A2B]" : "text-[#8B919C] hover:text-[#E8EAED]"
              }`}
              title="Toggle Grid Snapping"
            >
              <Magnet className="w-3 h-3" />
            </button>
          </div>

          {/* Render Mode Switcher */}
          <button
            onClick={toggleMode}
            className={`flex items-center gap-1 px-2 py-1 rounded border text-[11px] font-mono transition-colors duration-150 cursor-pointer ml-1.5 ${
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
              <span
                className={`w-2 h-2 rounded-full ${
                  playState === "playing"
                    ? "bg-[#3DD68C] animate-pulse"
                    : playState === "paused"
                    ? "bg-[#FFB020]"
                    : "bg-[#8B919C]"
                }`}
              />
              <span className="text-[#E8EAED]">
                {playState.toUpperCase()} · {fps.toFixed(1)} FPS
              </span>
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
          <div
            tabIndex={0}
            onPointerDown={handlePointerDown}
            onPointerMove={handlePointerMove}
            onPointerUp={handlePointerUp}
            onWheel={handleWheel}
            onKeyDown={handleKeyDownStream}
            onKeyUp={handleKeyUpStream}
            className="w-full h-full flex items-center justify-center bg-black overflow-hidden relative cursor-crosshair outline-none"
          >
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
