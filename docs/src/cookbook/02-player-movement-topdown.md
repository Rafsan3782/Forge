---
id: cookbook.player-movement-topdown
title: 8-Direction Top-Down Movement with Diagonal Normalization
summary: Responsive 8-direction top-down player controller with speed normalization and friction.
tags: ["cookbook", "topdown", "player", "movement", "normalization"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.game-feel-defaults", "cookbook.dash-mechanic"]
---

# Recipe: Top-Down 8-Direction Movement

## Goal
Implement responsive top-down character movement where diagonal traversal maintains equal speed to cardinal axes.

## Code

```typescript <!-- check -->
// scripts/player_topdown.ts
export const SPEED = 180.0;
let posX = 200.0;
let posY = 200.0;

export function onStart(): void {
  forge.log.info("Topdown movement script ready.");
}

export function onUpdate(dt: number): void {
  const safeDt = Math.min(dt, 0.05);

  let dx = 0.0;
  let dy = 0.0;

  if (forge.input.isActionPressed("move_left")) dx -= 1.0;
  if (forge.input.isActionPressed("move_right")) dx += 1.0;
  if (forge.input.isActionPressed("move_up")) dy += 1.0;
  if (forge.input.isActionPressed("move_down")) dy -= 1.0;

  if (dx !== 0.0 && dy !== 0.0) {
    const invSqrt2 = 0.70710678;
    dx *= invSqrt2;
    dy *= invSqrt2;
  }

  posX += dx * SPEED * safeDt;
  posY += dy * safeDt * SPEED;

  const transform = forge.entity.getTransform();
  transform.position[0] = posX;
  transform.position[1] = posY;
  forge.entity.setTransform(transform);
}
```

## How to Verify
1. Move diagonally (press `"move_right"` and `"move_up"`).
2. Verify total displacement after 1 second equals `180.0` pixels, not `254.5` pixels.

## Variations
- **Momentum & Inertia:** Use velocity vector with friction damping (`velX *= Math.pow(0.1, safeDt)`).
- **Facing Direction:** Rotate sprite towards mouse pointer via `forge.input.getMousePosition()`.
