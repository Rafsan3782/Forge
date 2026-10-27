---
id: cookbook.doors-keys
title: Locked Doors and Key Inventory Systems
summary: Collecting keys, unlocking blocked pathways, and audio cues for door access.
tags: ["cookbook", "doors", "keys", "puzzle", "inventory"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["cookbook.pickups-collectibles", "concepts.tilemaps"]
---

# Recipe: Doors & Keys

## Goal
Implement a key-pickup and locked door system that unlocks a blocked passage when approached with a key.

## Code

```typescript <!-- check -->
// scripts/doors_keys.ts
let hasKey = false;
let doorUnlocked = false;

const KEY_POS = [300.0, 80.0];
const DOOR_POS = [800.0, 80.0];
const INTERACT_DIST = 32.0;

export function updateDoorKeySystem(playerX: number, playerY: number, tilemapEntityId: string): void {
  // 1. Key Pickup Check
  if (!hasKey) {
    const dx = playerX - KEY_POS[0];
    const dy = playerY - KEY_POS[1];
    if (dx * dx + dy * dy < INTERACT_DIST * INTERACT_DIST) {
      hasKey = true;
      forge.audio.play("key.wav", { bus: "sfx" });
      forge.log.info("Key acquired!");
    }
  }

  // 2. Door Unlock Check
  if (hasKey && !doorUnlocked) {
    const dx = playerX - DOOR_POS[0];
    const dy = playerY - DOOR_POS[1];
    if (dx * dx + dy * dy < INTERACT_DIST * INTERACT_DIST) {
      doorUnlocked = true;
      // Clear door tile in tilemap
      forge.tilemap.setTile(tilemapEntityId, "main", 25, 2, 0);
      forge.audio.play("door_unlock.wav", { bus: "sfx" });
      forge.log.info("Door unlocked and opened!");
    }
  }
}
```

## How to Verify
1. Approach door without key -> door remains locked (`doorUnlocked === false`).
2. Approach key -> `hasKey` becomes `true`.
3. Approach door with key -> `doorUnlocked` becomes `true` and door tile clears.

## Variations
- **Multiple Keys:** Require `keysCollected === 3` before door unlocks.
- **Color Coded:** Track `hasRedKey`, `hasBlueKey` for distinct colored doors.
