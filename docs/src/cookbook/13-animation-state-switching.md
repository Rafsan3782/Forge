---
id: cookbook.animation-state-switching
title: Sprite Animation and Facing Direction Switching
summary: Procedural UV animation frame progression and horizontal flip direction switching.
tags: ["cookbook", "animation", "sprites", "flip", "animator"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["cookbook.player-movement-platformer", "concepts.rendering"]
---

# Recipe: Animation & Direction Switching

## Goal
Advance sprite animation frames based on movement speed and flip the sprite horizontally based on travel direction.

## Code

```typescript <!-- check -->
// scripts/animation_controller.ts
export const RUN_ANIM_FPS = 10.0;
export const TOTAL_FRAMES = 4;

let currentFrame = 0;
let frameTimer = 0.0;
let facingRight = true;

export function updateAnimation(dt: number, moveSpeedX: number): void {
  const isMoving = Math.abs(moveSpeedX) > 5.0;

  if (moveSpeedX > 0.0) facingRight = true;
  else if (moveSpeedX < 0.0) facingRight = false;

  if (isMoving) {
    frameTimer += dt;
    if (frameTimer >= 1.0 / RUN_ANIM_FPS) {
      frameTimer = 0.0;
      currentFrame = (currentFrame + 1) % TOTAL_FRAMES;
    }
  } else {
    currentFrame = 0;
    frameTimer = 0.0;
  }

  const sprite = forge.entity.getComponent<{ flip_x?: boolean }>("Sprite");
  if (sprite) {
    sprite.flip_x = !facingRight;
    forge.entity.setComponent("Sprite", sprite);
  }
}
```

## How to Verify
1. Move right -> verify `Sprite.flip_x` is `false`.
2. Move left -> verify `Sprite.flip_x` is `true`.
3. Stand still -> verify frame resets to `0`.

## Variations
- **Squash and Stretch:** Adjust scale `[0.85, 1.15]` during takeoff and landing frames.
- **Dust Trail:** Spawn a transient dust particle entity whenever `currentFrame` transitions while running.
