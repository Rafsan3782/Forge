---
id: cookbook.powerups-buffs
title: Temporary Powerups and Timed Buff Modifiers
summary: Speed boost, invulnerability star, and jump enhancement buff expiration timers.
tags: ["cookbook", "powerups", "buffs", "speed-boost", "timers"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["cookbook.pickups-collectibles", "agents.game-feel-defaults"]
---

# Recipe: Temporary Powerups & Buffs

## Goal
Apply temporary movement buffs (speed boost, enhanced jump) that expire after a set duration.

## Code

```typescript <!-- check -->
// scripts/powerups.ts
export const BASE_SPEED = 220.0;
export const BOOST_SPEED = 340.0;

let currentMoveSpeed = BASE_SPEED;
let speedBoostTimer = 0.0;

export function grantSpeedBoost(duration: number): void {
  speedBoostTimer = duration;
  currentMoveSpeed = BOOST_SPEED;
  forge.audio.play("powerup.wav", { bus: "sfx" });
  forge.log.info("Speed boost activated for " + duration + "s!");
}

export function onUpdate(dt: number): void {
  const safeDt = Math.min(dt, 0.05);

  if (speedBoostTimer > 0.0) {
    speedBoostTimer -= safeDt;
    if (speedBoostTimer <= 0.0) {
      currentMoveSpeed = BASE_SPEED;
      forge.log.info("Speed boost expired.");
    }
  }

  // Use currentMoveSpeed in player movement calculation
}
```

## How to Verify
1. Call `grantSpeedBoost(3.0)` -> `currentMoveSpeed` becomes `340.0`.
2. Advance 3.1s -> verify speed reverts to `220.0` and expiry log emitted.

## Variations
- **Super Jump:** Increase jump velocity by 30% while jump buff is active.
- **Sparkle Trails:** Emit particle burst every 0.1s while under powerup effect.
