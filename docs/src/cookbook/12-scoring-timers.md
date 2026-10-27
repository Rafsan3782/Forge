---
id: cookbook.scoring-timers
title: Scoring, Timers, and Combo Multipliers
summary: Elapsed level timers, coin scoring increments, and timed combo point decay.
tags: ["cookbook", "scoring", "timer", "combo", "gameplay"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["cookbook.pickups-collectibles", "agents.game-feel-defaults"]
---

# Recipe: Scoring & Level Timers

## Goal
Track elapsed level time, accumulate points, and reward rapid actions with combo multipliers.

## Code

```typescript <!-- check -->
// scripts/scoring.ts
let score = 0;
let levelTime = 0.0;
let comboMultiplier = 1.0;
let comboTimer = 0.0;
const COMBO_WINDOW = 2.0;

export function addPoints(basePoints: number): void {
  const points = Math.floor(basePoints * comboMultiplier);
  score += points;
  comboMultiplier = Math.min(4.0, comboMultiplier + 0.5);
  comboTimer = COMBO_WINDOW;
  forge.log.info(`Scored ${points} pts! Total: ${score} (Combo: x${comboMultiplier.toFixed(1)})`);
}

export function onUpdate(dt: number): void {
  const safeDt = Math.min(dt, 0.05);
  levelTime += safeDt;

  if (comboTimer > 0.0) {
    comboTimer -= safeDt;
    if (comboTimer <= 0.0) {
      comboMultiplier = 1.0;
      forge.log.info("Combo multiplier reset to 1.0");
    }
  }
}
```

## How to Verify
1. Call `addPoints(100)` -> score is `100`, combo is `x1.5`.
2. Call `addPoints(100)` immediately -> score is `250`, combo is `x2.0`.
3. Wait 2.1s -> verify combo resets to `1.0`.

## Variations
- **Time Attack:** Deduct points if `levelTime > TARGET_TIME`.
- **High Score Record:** Compare score against saved best upon level completion.
