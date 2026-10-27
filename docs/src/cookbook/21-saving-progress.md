---
id: cookbook.saving-progress
title: Saving and Restoring Simple Player Progress
summary: Serializing and deserializing score, unlocked levels, and checkpoint state.
tags: ["cookbook", "save", "load", "serialization", "progress"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["concepts.scripting", "cookbook.checkpoints-respawn"]
---

# Recipe: Saving Player Progress

## Goal
Implement state serialization and deserialization hooks to preserve progress across hot reloads and play sessions.

## Code

```typescript <!-- check -->
// scripts/save_system.ts
interface GameSaveData {
  highScore: number;
  unlockedLevel: number;
  totalCoins: number;
}

let progress: GameSaveData = {
  highScore: 0,
  unlockedLevel: 1,
  totalCoins: 0
};

export function recordScore(newScore: number): void {
  if (newScore > progress.highScore) {
    progress.highScore = newScore;
  }
}

// QuickJS hot reload lifecycle serialization hook
export function serialize(): unknown {
  return progress;
}

// QuickJS hot reload lifecycle restoration hook
export function deserialize(state: unknown): void {
  if (state && typeof state === "object") {
    progress = { ...progress, ...(state as GameSaveData) };
    forge.log.info("Progress restored: " + JSON.stringify(progress));
  }
}
```

## How to Verify
1. Set `progress.highScore = 500`.
2. Call `serialize()` -> verify returned object contains `highScore: 500`.
3. Call `deserialize({ highScore: 750, unlockedLevel: 2, totalCoins: 10 })` -> verify state merges correctly.

## Variations
- **Inventory Persistence:** Store array of collected item IDs in the save structure.
- **Save Slot Management:** Support multiple slot keys (`save_slot_1`, `save_slot_2`).
