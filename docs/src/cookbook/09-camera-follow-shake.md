---
id: cookbook.camera-follow-shake
title: Smooth Camera Follow and Trauma Screen Shake
summary: Smooth lerp camera tracking, viewport boundary clamping, and trauma-based screen shake.
tags: ["cookbook", "camera", "smooth-follow", "screen-shake", "game-feel"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.game-feel-defaults", "cookbook.health-damage"]
---

# Recipe: Camera Follow & Screen Shake

## Goal
Implement a smooth tracking camera that clamps to level boundaries and features decay-based screen shake on impacts.

## Code

```typescript <!-- check -->
// scripts/camera_controller.ts
export const SMOOTH_SPEED = 5.0;
export const MIN_X = 400.0;
export const MAX_X = 1200.0;
export const FIXED_Y = 150.0;

let currentCamX = MIN_X;
let currentCamY = FIXED_Y;
let trauma = 0.0; // 0.0 to 1.0
const TRAUMA_DECAY = 2.0;
const MAX_SHAKE_OFFSET = 12.0;

export function addTrauma(amount: number): void {
  trauma = Math.min(1.0, trauma + amount);
}

export function onUpdate(dt: number): void {
  const safeDt = Math.min(dt, 0.05);

  // Target coordinates (e.g. from player or level focus)
  const targetX = 600.0; 
  currentCamX += (targetX - currentCamX) * (1.0 - Math.exp(-SMOOTH_SPEED * safeDt));
  currentCamX = Math.max(MIN_X, Math.min(MAX_X, currentCamX));

  let offsetX = 0.0;
  let offsetY = 0.0;

  if (trauma > 0.0) {
    const shake = trauma * trauma; // Quadratic trauma feel
    offsetX = (Math.random() * 2.0 - 1.0) * MAX_SHAKE_OFFSET * shake;
    offsetY = (Math.random() * 2.0 - 1.0) * MAX_SHAKE_OFFSET * shake;
    trauma = Math.max(0.0, trauma - TRAUMA_DECAY * safeDt);
  }

  const transform = forge.entity.getTransform();
  transform.position[0] = currentCamX + offsetX;
  transform.position[1] = currentCamY + offsetY;
  forge.entity.setTransform(transform);
}
```

## How to Verify
1. Call `addTrauma(0.8)` -> observe camera jittering for ~0.4s.
2. Verify camera settles smoothly back to `currentCamX` without permanent drift.

## Variations
- **Directional Shake:** Bias shake offset along the direction of damage impact.
- **Lookahead Offset:** Shift target camera position 40px forward in hero's facing direction.
