---
id: cookbook.moving-platforms
title: Moving Platforms with Passenger Velocity Transfer
summary: Horizontal oscillating platforms that carry passengers standing on top of them.
tags: ["cookbook", "platforms", "moving-platform", "physics", "kinematics"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["cookbook.player-movement-platformer", "agents.game-feel-defaults"]
---

# Recipe: Moving Platforms

## Goal
Create an oscillating moving platform that carries the player along without slipping off.

## Code

```typescript <!-- check -->
// scripts/moving_platform.ts
export const MIN_X = 400.0;
export const MAX_X = 600.0;
export const SPEED = 60.0;
export const PLATFORM_Y = 160.0;
export const WIDTH = 96.0;
export const HEIGHT = 16.0;

let posX = MIN_X;
let dir = 1.0;
let velX = SPEED;

export function updatePlatform(dt: number): { x: number; y: number; vx: number } {
  const safeDt = Math.min(dt, 0.05);
  posX += dir * SPEED * safeDt;

  if (posX >= MAX_X) {
    posX = MAX_X;
    dir = -1.0;
  } else if (posX <= MIN_X) {
    posX = MIN_X;
    dir = 1.0;
  }

  velX = dir * SPEED;

  const transform = forge.entity.getTransform();
  transform.position[0] = posX;
  transform.position[1] = PLATFORM_Y;
  forge.entity.setTransform(transform);

  return { x: posX, y: PLATFORM_Y, vx: velX };
}
```

## How to Verify
1. Move platform for 5 seconds -> verify `posX` stays strictly between `400.0` and `600.0`.
2. Ground player on platform top -> player `posX` shifts by `velX * dt`.

## Variations
- **Vertical Elevator:** Oscillate along the Y-axis between `80.0` and `240.0`.
- **Triggered Lift:** Only begin moving once the player steps onto the surface.
