---
id: cookbook.pausing-game
title: Pausing Game Simulation and Modal State
summary: Freezing gameplay movement and physics updates while maintaining menu input responsiveness.
tags: ["cookbook", "pausing", "simulation", "menu", "state"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["cookbook.state-machines", "concepts.input"]
---

# Recipe: Game Pausing

## Goal
Freeze gameplay entity updates when pressing the pause button while retaining input polling for pause menu options.

## Code

```typescript <!-- check -->
// scripts/pause_controller.ts
let isPaused = false;

export function togglePause(): boolean {
  isPaused = !isPaused;
  forge.audio.setBusVolume("sfx", isPaused ? 0.3 : 0.8);
  forge.log.info(isPaused ? "Game Paused." : "Game Resumed.");
  return isPaused;
}

export function onUpdate(dt: number): void {
  if (forge.input.isActionJustPressed("pause")) {
    togglePause();
  }

  if (isPaused) {
    // In pause state: do not integrate player/enemy positions or timers
    return;
  }

  // Normal gameplay updates continue below
  const safeDt = Math.min(dt, 0.05);
  const transform = forge.entity.getTransform();
  transform.position[0] += 50.0 * safeDt;
  forge.entity.setTransform(transform);
}
```

## How to Verify
1. Press `"pause"` -> verify log output `"Game Paused."`.
2. Move hero inputs -> verify transform position does not change while paused.
3. Press `"pause"` again -> verify position begins advancing.

## Variations
- **Slow Motion:** Instead of completely pausing, multiply delta time by `0.2`.
- **Audio Low-Pass Simulation:** Reduce master volume and duck music during pause.
