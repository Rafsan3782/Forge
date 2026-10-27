---
id: cookbook.game-over-victory
title: Game Over and Level Victory Sequences
summary: Win and loss conditions, score tally evaluation, and session restart hooks.
tags: ["cookbook", "game-over", "victory", "session", "loop"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["cookbook.scoring-timers", "cookbook.scene-transitions"]
---

# Recipe: Game Over & Victory Sequences

## Goal
Manage game over and victory game states, lock simulation controls, trigger thematic music, and allow clean restarts.

## Code

```typescript <!-- check -->
// scripts/game_flow.ts
type FlowState = "playing" | "victory" | "game_over";

let flowState: FlowState = "playing";
let stateTimer = 0.0;

export function triggerVictory(): void {
  if (flowState !== "playing") return;
  flowState = "victory";
  stateTimer = 0.0;
  forge.audio.stop(undefined, "music");
  forge.audio.play("victory.wav", { bus: "music" });
  forge.log.info("VICTORY! All objectives completed.");
}

export function triggerGameOver(): void {
  if (flowState !== "playing") return;
  flowState = "game_over";
  stateTimer = 0.0;
  forge.audio.stop(undefined, "music");
  forge.audio.play("gameover.wav", { bus: "sfx" });
  forge.log.info("GAME OVER. Press Jump to restart.");
}

export function onUpdate(dt: number): void {
  stateTimer += dt;

  if (flowState === "game_over") {
    if (stateTimer > 1.0 && forge.input.isActionJustPressed("jump")) {
      forge.scene.load("scenes/level_1.json");
    }
    return;
  }

  if (flowState === "victory") {
    if (stateTimer > 2.0 && forge.input.isActionJustPressed("jump")) {
      forge.scene.load("scenes/level_2.json");
    }
    return;
  }
}
```

## How to Verify
1. Call `triggerGameOver()` -> verify music stops and `"gameover.wav"` plays.
2. Press jump before 1.0s -> nothing happens.
3. Press jump after 1.0s -> verify reload of `scenes/level_1.json`.

## Variations
- **Score Bonus Calculation:** Add remaining time * 10 to final score on victory.
- **Defeat Animation:** Spin hero sprite and drop through floor on game over.
