---
id: cookbook.parallax-backgrounds
title: Multi-Layer Parallax Background Scrolling
summary: Scrolling background sprite layers at fractional speeds relative to the main camera.
tags: ["cookbook", "parallax", "background", "camera", "visuals"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["concepts.rendering", "cookbook.camera-follow-shake"]
---

# Recipe: Parallax Backgrounds

## Goal
Scroll multiple background layers at fractional speeds relative to the camera to create visual depth.

## Code

```typescript <!-- check -->
// scripts/parallax.ts
interface ParallaxLayer {
  entityId: string;
  scrollFactor: number; // e.g. 0.2 for distant clouds, 0.5 for hills
  initialX: number;
}

const layers: ParallaxLayer[] = [
  { entityId: "bg_mountains", scrollFactor: 0.2, initialX: 0.0 },
  { entityId: "bg_trees", scrollFactor: 0.5, initialX: 0.0 }
];

export function updateParallax(cameraX: number): void {
  for (const layer of layers) {
    const layerX = layer.initialX + cameraX * (1.0 - layer.scrollFactor);
    // In actual implementation, setTransform on each background entity
    forge.log.info(`Layer ${layer.entityId} positioned at x=${layerX}`);
  }
}
```

## How to Verify
1. Camera moves right by `100.0` pixels.
2. Verify distant mountains move by only `80.0` px in relative parallax terms.

## Variations
- **Vertical Parallax:** Apply separate vertical scroll factor (e.g. `0.1` for sky).
- **Infinite Texture Repeat:** Offset UV texture coordinates instead of moving entities.
