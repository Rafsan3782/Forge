---
id: cookbook.pickups-collectibles
title: Collectible Pickups and Inventory Counts
summary: Coin and item pickup collision detection, audio feedback, scoring, and entity cleanup.
tags: ["cookbook", "pickups", "coins", "collectibles", "inventory"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["cookbook.scoring-timers", "concepts.scripting"]
---

# Recipe: Collectibles & Pickups

## Goal
Detect player overlap with collectible items (coins, gems), increment score, play audio cues, and remove the picked-up item.

## Code

```typescript <!-- check -->
// scripts/pickups.ts
interface Collectible {
  id: string;
  x: number;
  y: number;
  value: number;
  collected: boolean;
}

const PICKUP_RADIUS = 24.0;
let totalCoins = 0;

const coinsInLevel: Collectible[] = [
  { id: "coin_1", x: 200, y: 80, value: 1, collected: false },
  { id: "coin_2", x: 350, y: 120, value: 1, collected: false },
  { id: "gem_1", x: 500, y: 160, value: 5, collected: false },
];

export function onUpdate(_dt: number): void {
  const heroTransform = forge.entity.getTransform();
  const heroX = heroTransform.position[0];
  const heroY = heroTransform.position[1];

  for (const c of coinsInLevel) {
    if (c.collected) continue;

    const dx = heroX - c.x;
    const dy = heroY - c.y;
    const distSq = dx * dx + dy * dy;

    if (distSq < PICKUP_RADIUS * PICKUP_RADIUS) {
      c.collected = true;
      totalCoins += c.value;
      forge.audio.play("coin.wav", { bus: "sfx", volume: 0.8 });
      forge.log.info(`Collected item ${c.id}! New total coins: ${totalCoins}`);
    }
  }
}
```

## How to Verify
1. Position player at `[200.0, 80.0]`.
2. Verify `coin_1` is marked `collected = true` and `totalCoins` increments to `1`.
3. Stand at same position on subsequent frames -> verify no duplicate increments occur.

## Variations
- **Bobbing Animation:** Add sinusoidal vertical offset `y = baseY + Math.sin(time * 4.0) * 4.0`.
- **Magnetic Attraction:** When player is within 64px, smoothly pull coin toward player position.
