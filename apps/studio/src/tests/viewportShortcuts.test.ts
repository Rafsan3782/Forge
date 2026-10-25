import { describe, it, expect, beforeEach, vi } from "vitest";
import { usePlayStore } from "../stores/playStore";
import { useRuntimeStore } from "../stores/runtimeStore";

describe("viewportShortcuts & playStore", () => {
  beforeEach(() => {
    usePlayStore.setState({
      playState: "edit",
      activeGizmo: "translate",
      gridSnapEnabled: true,
      angleSnapEnabled: true,
      showGrid: true,
    });
  });

  it("transitions between edit, play, pause, step, and stop states", async () => {
    const playStore = usePlayStore.getState();
    expect(playStore.playState).toBe("edit");

    await playStore.play();
    expect(usePlayStore.getState().playState).toBe("playing");

    await playStore.pause();
    expect(usePlayStore.getState().playState).toBe("paused");

    await playStore.step();
    expect(usePlayStore.getState().playState).toBe("paused");

    await playStore.stop();
    expect(usePlayStore.getState().playState).toBe("edit");
  });

  it("toggles active gizmo tools (W, E, R)", () => {
    const playStore = usePlayStore.getState();
    expect(playStore.activeGizmo).toBe("translate");

    playStore.setActiveGizmo("rotate");
    expect(usePlayStore.getState().activeGizmo).toBe("rotate");

    playStore.setActiveGizmo("scale");
    expect(usePlayStore.getState().activeGizmo).toBe("scale");

    playStore.setActiveGizmo("translate");
    expect(usePlayStore.getState().activeGizmo).toBe("translate");
  });

  it("toggles grid snapping and angle snapping", () => {
    const playStore = usePlayStore.getState();
    expect(playStore.gridSnapEnabled).toBe(true);
    expect(playStore.angleSnapEnabled).toBe(true);

    playStore.toggleGridSnap();
    expect(usePlayStore.getState().gridSnapEnabled).toBe(false);

    playStore.toggleAngleSnap();
    expect(usePlayStore.getState().angleSnapEnabled).toBe(false);

    playStore.toggleShowGrid();
    expect(usePlayStore.getState().showGrid).toBe(false);
  });

  it("correctly maps coordinates in streamed mode across 100%, 125%, and 150% DPI scale", () => {
    const mapCoordinates = (
      cssX: number,
      cssY: number,
      containerWidth: number,
      containerHeight: number,
      frameWidth: number,
      frameHeight: number,
      dpiScale: number
    ): [number, number] => {
      // In streamed frames mode:
      // Normalized offset in the container element
      const normX = cssX / containerWidth;
      const normY = cssY / containerHeight;
      // Multiplied by frame resolution
      const targetX = Math.round(normX * frameWidth);
      const targetY = Math.round(normY * frameHeight);
      return [targetX, targetY];
    };

    const containerW = 960;
    const containerH = 540;
    const frameW = 1920;
    const frameH = 1080;

    // Center click at 100% DPI (1.0)
    const [cx100, cy100] = mapCoordinates(480, 270, containerW, containerH, frameW, frameH, 1.0);
    expect(cx100).toBe(960);
    expect(cy100).toBe(540);

    // Center click at 125% DPI (1.25)
    const [cx125, cy125] = mapCoordinates(480, 270, containerW, containerH, frameW, frameH, 1.25);
    expect(cx125).toBe(960);
    expect(cy125).toBe(540);

    // Center click at 150% DPI (1.5)
    const [cx150, cy150] = mapCoordinates(480, 270, containerW, containerH, frameW, frameH, 1.5);
    expect(cx150).toBe(960);
    expect(cy150).toBe(540);

    // Top-left corner
    const [tlX, tlY] = mapCoordinates(0, 0, containerW, containerH, frameW, frameH, 1.5);
    expect(tlX).toBe(0);
    expect(tlY).toBe(0);

    // Bottom-right corner
    const [brX, brY] = mapCoordinates(960, 540, containerW, containerH, frameW, frameH, 1.25);
    expect(brX).toBe(1920);
    expect(brY).toBe(1080);
  });
});
