---
id: cookbook-procedural-tilemap
title: "Procedural Tilemap Generation"
summary: "Recipe for generating procedural terrain and collision tiles using the forge.tilemap script API."
tags: ["cookbook", "recipe", "tilemap", "procedural"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["ref-comp-tilemap", "ref-tool-tilemap-fill_rect", "ref-tool-tilemap-set_tile"]
---

# Procedural Tilemap Generation

This recipe shows how to generate platform levels dynamically by placing ground, wall, and decorative tiles.

---

## Script (`scripts/terrain_generator.ts`)

```snippet
export function onStart(): void {
  const mapEntityId = forge.entity.getId();
  const layer = "main";
  const groundTileId = 1;
  const grassTileId = 2;

  // Generate ground terrain
  for (let x = 0; x < 40; x++) {
    // Solid underground
    forge.tilemap.setTile(mapEntityId, layer, x, 0, groundTileId);
    forge.tilemap.setTile(mapEntityId, layer, x, 1, groundTileId);
    // Surface grass
    forge.tilemap.setTile(mapEntityId, layer, x, 2, grassTileId);
  }

  // Create raised floating platforms
  for (let px = 10; px < 16; px++) {
    forge.tilemap.setTile(mapEntityId, layer, px, 6, grassTileId);
  }

  forge.log.info("Procedural level terrain generated.");
}
```

---

## Best Practices

- Use `tilemap.fill_rect` bridge tool for large block batches before runtime starts to minimize individual draw updates.
- Set tile ID `0` to erase tiles and reclaim memory chunks.
