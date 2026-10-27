---
id: cookbook.spawners-pools
title: Entity Spawners and Object Wave Timers
summary: Timed entity spawning, wave countdowns, capacity clamping, and reuse.
tags: ["cookbook", "spawner", "wave", "timer", "pool"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["cookbook.shooting-projectiles", "concepts.scripting"]
---

# Recipe: Spawners & Wave Timers

## Goal
Spawn enemy or item waves periodically on a timer up to a maximum concurrent entity threshold.

## Code

```typescript <!-- check -->
// scripts/spawner.ts
export const SPAWN_INTERVAL = 2.5;
export const MAX_CONCURRENT = 4;

let timer = 0.0;
let waveCount = 0;
const activeEntities: string[] = [];

export function onUpdate(dt: number): void {
  const safeDt = Math.min(dt, 0.05);
  timer += safeDt;

  if (timer >= SPAWN_INTERVAL) {
    timer -= SPAWN_INTERVAL;

    if (activeEntities.length < MAX_CONCURRENT) {
      waveCount += 1;
      const entityId = forge.spawn(`WaveEnemy_${waveCount}`, {
        Transform2D: { position: [700.0, 80.0], rotation: 0, scale: [1, 1] },
        Sprite: { asset_id: "patroller.png", z_index: 5 }
      });
      activeEntities.push(entityId);
      forge.log.info(`Spawned enemy: ${entityId}. Active total: ${activeEntities.length}`);
    }
  }
}
```

## How to Verify
1. Run simulation for 11 seconds.
2. Confirm 4 entities are spawned.
3. Confirm no 5th entity spawns while `activeEntities.length >= MAX_CONCURRENT`.

## Variations
- **Random Location:** Pick `spawnX` randomly between two horizontal boundaries.
- **Progressive Difficulty:** Decrease `SPAWN_INTERVAL` by 5% with each successive wave.
