---
id: cookbook.tilemap-level-building
title: Procedural Tilemap Level Generation
summary: Building level floors, walls, and platforms via script and bridge tilemap operations.
tags: ["cookbook", "tilemap", "level-building", "procedural", "generation"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["concepts.tilemaps", "agents.playbooks"]
---

# Recipe: Procedural Tilemap Generation

## Goal
Generate level platforms and obstacle layouts dynamically using `forge.tilemap` script APIs.

## Code

```typescript <!-- check -->
// scripts/level_builder.ts
export function buildLevel(tilemapEntityId: string): void {
  // Clear ground layer
  const LAYER = "main";
  const TILE_SOLID = 1;
  const TILE_PLATFORM = 2;

  // Build main floor span from x=0 to x=20 at y=0
  for (let x = 0; x < 20; x++) {
    forge.tilemap.setTile(tilemapEntityId, LAYER, x, 0, TILE_SOLID);
    forge.tilemap.setTile(tilemapEntityId, LAYER, x, 1, TILE_SOLID);
  }

  // Build elevated floating platform at x=8..14, y=4
  for (let x = 8; x <= 14; x++) {
    forge.tilemap.setTile(tilemapEntityId, LAYER, x, 4, TILE_PLATFORM);
  }

  forge.log.info("Procedural level construction completed.");
}
```

## How to Verify
1. Execute `buildLevel("tilemap")`.
2. Inspect `tilemap.get("tilemap", "main", 10, 4)` -> verify returned tile ID is `2`.
3. Inspect `tilemap.get("tilemap", "main", 5, 0)` -> verify returned tile ID is `1`.

## Variations
- **Pitfalls:** Skip ground generation between `x=12` and `x=15` to create a jump gap.
- **Random Obstacles:** Place hazard spikes with 15% probability on solid ground tiles.
