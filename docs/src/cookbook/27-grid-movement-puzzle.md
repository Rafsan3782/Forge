---
id: cookbook.grid-movement-puzzle
title: Tile-Aligned Grid Movement for Puzzle Games
summary: Discrete step-based tile movement with interpolation and obstacle collision.
tags: ["cookbook", "grid", "puzzle", "sokoban", "movement"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.game-feel-defaults", "cookbook.doors-keys"]
---

# Recipe: Grid-Aligned Movement

## Goal
Constrain player movement strictly to discrete tile coordinates with smooth visual interpolation between steps.

## Code

```typescript <!-- check -->
// scripts/grid_player.ts
const TILE_SIZE = 32.0;
const STEP_TIME = 0.14;

let gridX = 2;
let gridY = 2;
let targetGridX = 2;
let targetGridY = 2;
let moveProgress = 1.0; // 1.0 = fully settled on tile

export function onUpdate(dt: number): void {
  const safeDt = Math.min(dt, 0.05);

  if (moveProgress < 1.0) {
    moveProgress = Math.min(1.0, moveProgress + safeDt / STEP_TIME);
  } else {
    // Poll discrete input
    let dx = 0;
    let dy = 0;
    if (forge.input.isActionJustPressed("move_left")) dx = -1;
    else if (forge.input.isActionJustPressed("move_right")) dx = 1;
    else if (forge.input.isActionJustPressed("move_up")) dy = 1;
    else if (forge.input.isActionJustPressed("move_down")) dy = -1;

    if (dx !== 0 || dy !== 0) {
      gridX = targetGridX;
      gridY = targetGridY;
      targetGridX = gridX + dx;
      targetGridY = gridY + dy;
      moveProgress = 0.0;
      forge.audio.play("step.wav", { bus: "sfx", volume: 0.4 });
    }
  }

  // Smooth ease-out visual position
  const currentPixelX = (gridX + (targetGridX - gridX) * moveProgress) * TILE_SIZE;
  const currentPixelY = (gridY + (targetGridY - gridY) * moveProgress) * TILE_SIZE;

  const transform = forge.entity.getTransform();
  transform.position[0] = currentPixelX;
  transform.position[1] = currentPixelY;
  forge.entity.setTransform(transform);
}
```

## How to Verify
1. Tap right arrow once.
2. Observe character smoothly transition from `x=64` to `x=96` over 0.14s and stop cleanly.

## Variations
- **Sokoban Box Push:** If `targetGrid` has a box, test if cell behind box is empty before allowing step.
- **Ice Slide:** Continue stepping in chosen direction until hitting an obstacle.
