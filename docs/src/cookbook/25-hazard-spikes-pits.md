---
id: cookbook.hazard-spikes-pits
title: Hazards, Spikes, and Bottomless Pits
summary: Immediate death and knockback volumes with pitfall respawn triggers.
tags: ["cookbook", "hazards", "spikes", "pits", "death"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["cookbook.checkpoints-respawn", "cookbook.health-damage"]
---

# Recipe: Hazards & Bottomless Pits

## Goal
Detect player contact with stationary spikes or falling into bottomless gaps and trigger immediate damage and respawn.

## Code

```typescript <!-- check -->
// scripts/hazards.ts
interface HazardRect {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
}

const SPIKES: HazardRect[] = [
  { minX: 672, minY: 64, maxX: 720, maxY: 88 },
  { minX: 928, minY: 64, maxX: 976, maxY: 88 },
];

const PIT_KILL_Y = -50.0;

export function checkHazards(posX: number, posY: number): boolean {
  // 1. Pitfall check
  if (posY < PIT_KILL_Y) {
    forge.log.info("Player fell into the pit!");
    return true;
  }

  // 2. AABB Spikes overlap
  const playerWidth = 24.0;
  const playerHeight = 32.0;

  for (const h of SPIKES) {
    if (
      posX + playerWidth / 2 > h.minX &&
      posX - playerWidth / 2 < h.maxX &&
      posY + playerHeight > h.minY &&
      posY < h.maxY
    ) {
      forge.log.info("Player struck spikes!");
      return true;
    }
  }

  return false;
}
```

## How to Verify
1. Position player at `[690.0, 70.0]` -> `checkHazards` returns `true`.
2. Drop player below `y = -50.0` -> `checkHazards` returns `true`.

## Variations
- **Retractable Spikes:** Toggle hazard active state on a 2-second on/off cycle.
- **Lava Floor:** Instant death with bubbling particle burst upon touching bottom liquid boundary.
