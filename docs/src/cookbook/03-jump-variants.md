---
id: cookbook.jump-variants
title: Double Jump and Wall Jump Mechanics
summary: Advanced platformer jump variants including air double jump and wall kick mechanics.
tags: ["cookbook", "platformer", "jump", "double-jump", "wall-jump"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["cookbook.player-movement-platformer", "agents.game-feel-defaults"]
---

# Recipe: Double Jump & Wall Jump

## Goal
Add air double jump and directional wall kick capabilities to a 2D platformer character.

## Code

```typescript <!-- check -->
// scripts/jump_variants.ts
export const JUMP_VELOCITY = 420.0;
export const DOUBLE_JUMP_VELOCITY = 380.0;
export const WALL_JUMP_VEL_X = 260.0;

let velX = 0.0;
let velY = 0.0;
let isGrounded = true;
let canDoubleJump = true;
let isTouchingWall = false;
let wallNormalX = -1.0; // 1.0 means wall is to the left, pushing player right

export function onUpdate(dt: number): void {
  const safeDt = Math.min(dt, 0.05);

  if (isGrounded) {
    canDoubleJump = true;
  }

  if (forge.input.isActionJustPressed("jump")) {
    if (isGrounded) {
      velY = JUMP_VELOCITY;
      isGrounded = false;
      forge.audio.play("jump.wav", { bus: "sfx" });
    } else if (isTouchingWall) {
      velY = JUMP_VELOCITY;
      velX = wallNormalX * WALL_JUMP_VEL_X;
      forge.audio.play("walljump.wav", { bus: "sfx" });
    } else if (canDoubleJump) {
      velY = DOUBLE_JUMP_VELOCITY;
      canDoubleJump = false;
      forge.audio.play("doublejump.wav", { bus: "sfx" });
    }
  }

  // Simple downward gravity simulation
  if (!isGrounded) {
    velY -= 980.0 * safeDt;
  }

  const transform = forge.entity.getTransform();
  transform.position[0] += velX * safeDt;
  transform.position[1] += velY * safeDt;
  forge.entity.setTransform(transform);
}
```

## How to Verify
1. Jump once while in the air -> confirm `velY` resets to `DOUBLE_JUMP_VELOCITY`.
2. Jump a third time in the air -> confirm no further jump occurs until ground contact.

## Variations
- **Triple Jump:** Use integer counter `jumpsRemaining = 2`.
- **Wall Slide:** Clamp negative `velY` to `-80.0` while `isTouchingWall` is true.
