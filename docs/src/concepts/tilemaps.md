---
id: concepts.tilemaps
title: Chunked 2D Tilemap System
summary: Chunked tilemap grid system, multi-layer rendering, and dynamic tile manipulation.
tags: ["concepts", "tilemaps", "chunks", "level-design"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.conventions", "agents.playbooks"]
---

# Chunked Tilemap System

Forge features an efficient chunked 2D tilemap system designed for fast rendering and real-time editing.

## Architecture

- **Chunking:** Tilemaps are divided into chunks (typically 16x16 tiles) to support large and streaming worlds without rendering offscreen geometry.
- **Multi-layer:** Supports arbitrary named layers (e.g. `background`, `ground`, `foreground`, `decorations`).
- **Palettes:** `tileset.json` maps integer tile IDs to texture atlas UV coordinates.
- **Bridge & Script Operations:**
  - `tilemap.get(entityId, layer, x, y)`: Read tile ID.
  - `tilemap.set_tile(entityId, layer, x, y, tileId)`: Set single tile.
  - `tilemap.fill_rect(entityId, layer, bounds, tileId)`: Bulk paint rectangular region.
  - `tilemap.clear(entityId, layer)`: Clear layer tiles.
