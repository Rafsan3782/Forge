---
id: cookbook.player-movement-platformer
title: Platformer Kinematic Movement with Coyote Time and Buffering
summary: Complete platformer player controller with horizontal acceleration, variable jump, coyote time, and jump buffering.
tags: ["cookbook", "platformer", "player", "movement", "jump", "coyote-time"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.game-feel-defaults", "cookbook.camera-follow-shake"]
---

# Recipe: Platformer Kinematic Movement

## Goal
Implement a responsive 2D platformer character controller featuring horizontal movement, gravity, variable jump height, coyote time, and jump buffering.

## Code

```typescript <!-- check -->
// scripts/player_platformer.ts
export const MOVE_SPEED = 220.0;
export const JUMP_VELOCITY = 440.0;
export const GRAVITY = 980.0;
export const MAX_FALL_SPEED = 550.0;
export const COYOTE_TIME = 0.12;
export const JUMP_BUFFER_TIME = 0.12;
export const VARIABLE_JUMP_FALL_MULT = 2.2;

let posX = 64.0;
let posY = 80.0;
let velX = 0.0;
let velY = 0.0;
let isGrounded = true;
let coyoteTimer = 0.0;
let jumpBufferTimer = 0.0;

export function onStart(): void {
  forge.log.info("Platformer player script loaded.");
}

export function onUpdate(dt: number): void {
  const safeDt = Math.min(dt, 0.05);

  let inputX = forge.input.getAxis("move_x");
  if (forge.input.isActionPressed("move_left")) inputX = -1.0;
  if (forge.input.isActionPressed("move_right")) inputX = 1.0;
  velX = inputX * MOVE_SPEED;

  if (isGrounded) {
    coyoteTimer = COYOTE_TIME;
  } else {
    coyoteTimer = Math.max(0.0, coyoteTimer - safeDt);
  }

  if (forge.input.isActionJustPressed("jump")) {
    jumpBufferTimer = JUMP_BUFFER_TIME;
  } else {
    jumpBufferTimer = Math.max(0.0, jumpBufferTimer - safeDt);
  }

  if (jumpBufferTimer > 0.0 && coyoteTimer > 0.0) {
    velY = JUMP_VELOCITY;
    isGrounded = false;
    coyoteTimer = 0.0;
    jumpBufferTimer = 0.0;
    forge.audio.play("jump.wav", { bus: "sfx", volume: 0.8 });
  }

  if (!isGrounded) {
    let currentGravity = GRAVITY;
    if (velY > 0.0 && !forge.input.isActionPressed("jump")) {
      currentGravity *= VARIABLE_JUMP_FALL_MULT;
    }
    velY -= currentGravity * safeDt;
    velY = Math.max(velY, -MAX_FALL_SPEED);
  }

  posX += velX * safeDt;
  posY += velY * safeDt;

  if (posY <= 80.0) {
    posY = 80.0;
    velY = 0.0;
    isGrounded = true;
  }

  const transform = forge.entity.getTransform();
  transform.position[0] = posX;
  transform.position[1] = posY;
  forge.entity.setTransform(transform);
}
```

## How to Verify
1. Attach script to Player entity with `Transform2D` at `[64.0, 80.0]`.
2. Run headless test: simulate `"move_right"` and `"jump"`.
3. Check `game.log` for jump sound and verify `posY` reaches apex ~178px.

## Variations
- **Ice / Slippery Surface:** Replace instant `velX` with linear acceleration (`velX = lerp(velX, targetVelX, 4.0 * safeDt)`).
- **Wall Slide:** When colliding with a wall while falling, clamp downward fall speed to `120.0 px/s`.
