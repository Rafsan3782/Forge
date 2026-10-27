---
id: cookbook.dash-mechanic
title: High-Speed Dash and Dodge Roll Mechanics
summary: Directional quick dash burst with input locks and recovery cooldowns.
tags: ["cookbook", "dash", "dodge", "movement", "game-feel"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.game-feel-defaults", "cookbook.player-movement-topdown"]
---

# Recipe: Dash Mechanic

## Goal
Provide a high-speed dash burst in the facing direction with a brief cooldown to prevent continuous dashing.

## Code

```typescript <!-- check -->
// scripts/dash.ts
export const DASH_SPEED = 450.0;
export const DASH_DURATION = 0.15;
export const DASH_COOLDOWN = 0.6;

let dashTimer = 0.0;
let cooldownTimer = 0.0;
let dashDirX = 1.0;

export function onUpdate(dt: number): void {
  const safeDt = Math.min(dt, 0.05);

  if (cooldownTimer > 0.0) cooldownTimer -= safeDt;

  if (forge.input.isActionJustPressed("dash") && cooldownTimer <= 0.0 && dashTimer <= 0.0) {
    dashTimer = DASH_DURATION;
    cooldownTimer = DASH_COOLDOWN;
    forge.audio.play("dash.wav", { bus: "sfx" });
  }

  const transform = forge.entity.getTransform();

  if (dashTimer > 0.0) {
    dashTimer -= safeDt;
    transform.position[0] += dashDirX * DASH_SPEED * safeDt;
    forge.entity.setTransform(transform);
  }
}
```

## How to Verify
1. Press `"dash"` -> verify velocity bursts for 0.15s covering ~67 pixels.
2. Press `"dash"` immediately again -> verify dash does not trigger during 0.6s cooldown.

## Variations
- **Ghost Trail:** Emit stationary shadow sprites every 0.03s during the dash duration.
- **Invulnerable Dash:** Grant invulnerability frames while `dashTimer > 0.0`.
