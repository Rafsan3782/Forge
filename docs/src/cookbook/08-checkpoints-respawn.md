---
id: cookbook.checkpoints-respawn
title: Checkpoints and Player Respawn Systems
summary: Checkpoint trigger detection, saving active spawn coordinates, and clean player death respawning.
tags: ["cookbook", "checkpoints", "respawn", "level-design"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["cookbook.health-damage", "agents.the-work-loop"]
---

# Recipe: Checkpoints & Respawn

## Goal
Save progressive respawn locations when reaching milestone checkpoints and reset player state on hazard death.

## Code

```typescript <!-- check -->
// scripts/respawn_system.ts
const INITIAL_SPAWN_X = 64.0;
const INITIAL_SPAWN_Y = 80.0;
const CHECKPOINT_RADIUS = 32.0;

let currentSpawnX = INITIAL_SPAWN_X;
let currentSpawnY = INITIAL_SPAWN_Y;

interface Checkpoint {
  id: string;
  x: number;
  y: number;
  activated: boolean;
}

const checkpoints: Checkpoint[] = [
  { id: "cp_midpoint", x: 600.0, y: 80.0, activated: false }
];

export function checkCheckpoints(playerX: number, playerY: number): void {
  for (const cp of checkpoints) {
    if (cp.activated) continue;
    const dx = playerX - cp.x;
    const dy = playerY - cp.y;
    if (dx * dx + dy * dy < CHECKPOINT_RADIUS * CHECKPOINT_RADIUS) {
      cp.activated = true;
      currentSpawnX = cp.x;
      currentSpawnY = cp.y;
      forge.audio.play("checkpoint.wav", { bus: "sfx" });
      forge.log.info("Checkpoint activated at x=" + cp.x);
    }
  }
}

export function respawnPlayer(): void {
  const transform = forge.entity.getTransform();
  transform.position[0] = currentSpawnX;
  transform.position[1] = currentSpawnY;
  forge.entity.setTransform(transform);
  forge.log.info("Player respawned at [" + currentSpawnX + ", " + currentSpawnY + "]");
}
```

## How to Verify
1. Move past `[600.0, 80.0]` -> check log for `"Checkpoint activated"`.
2. Call `respawnPlayer()` -> verify transform position snaps to `[600.0, 80.0]` instead of `[64.0, 80.0]`.

## Variations
- **Visual Flag Pole:** Animate or swap checkpoint sprite from red flag to green flag when activated.
- **Fall-out-of-bounds:** Automatically trigger `respawnPlayer()` whenever player `posY < -100.0`.
