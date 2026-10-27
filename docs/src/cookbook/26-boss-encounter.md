---
id: cookbook.boss-encounter
title: Boss Fight Pattern and Phased Attacks
summary: Multi-phase boss encounter with state-driven telegraphing, charge attacks, and vulnerable phases.
tags: ["cookbook", "boss", "combat", "phases", "ai"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["cookbook.state-machines", "cookbook.health-damage"]
---

# Recipe: Multi-Phase Boss Encounter

## Goal
Implement a multi-phase boss encounter cycling through telegraph, charge attack, and vulnerable cooldown states.

## Code

```typescript <!-- check -->
// scripts/boss_ai.ts
type BossPhase = "idle" | "telegraph" | "charge" | "vulnerable";

let phase: BossPhase = "idle";
let phaseTimer = 2.0;
let bossHealth = 5;
let bossX = 1000.0;
const CHARGE_SPEED = 300.0;

export function onUpdate(dt: number): void {
  const safeDt = Math.min(dt, 0.05);
  phaseTimer -= safeDt;

  switch (phase) {
    case "idle":
      if (phaseTimer <= 0.0) {
        phase = "telegraph";
        phaseTimer = 1.0;
        forge.audio.play("boss_roar.wav", { bus: "sfx" });
      }
      break;

    case "telegraph":
      if (phaseTimer <= 0.0) {
        phase = "charge";
        phaseTimer = 1.5;
      }
      break;

    case "charge":
      bossX -= CHARGE_SPEED * safeDt;
      if (phaseTimer <= 0.0) {
        phase = "vulnerable";
        phaseTimer = 2.5; // Player opening to strike
        forge.audio.play("boss_tired.wav", { bus: "sfx" });
      }
      break;

    case "vulnerable":
      if (phaseTimer <= 0.0) {
        phase = "idle";
        phaseTimer = 2.0;
      }
      break;
  }
}
```

## How to Verify
1. Observe sequence: `"idle"` (2s) -> `"telegraph"` (1s) -> `"charge"` (1.5s) -> `"vulnerable"` (2.5s).
2. Damage dealt during `"vulnerable"` decrements `bossHealth`.

## Variations
- **Phase 2 Enrage:** When `bossHealth <= 2`, double `CHARGE_SPEED` and halve `phaseTimer`.
- **Shockwave Impact:** Emit ground shockwaves when boss reaches end of charge.
