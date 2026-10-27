---
id: concepts.rendering
title: 2D WGPU Rendering Pipeline
summary: Architecture of the batch sprite, tilemap, and offscreen camera rendering system.
tags: ["concepts", "rendering", "wgpu", "sprites", "tilemaps"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.conventions", "concepts.scenes"]
---

# 2D Rendering Pipeline

The Forge rendering pipeline is implemented in Rust using modern `wgpu` primitives for cross-platform GPU performance.

## Key Subsystems

- **RenderContext:** Manages WGPU device, adapter, command queues, and swapchain surfaces.
- **SpriteBatchRenderer:** Collects all visible `Sprite` components, sorts them by `z_index`, batches vertices into unified dynamic buffers, and binds texture atlases.
- **TilemapRenderer:** Renders chunked grid layers with vertex instancing, handling multi-layer opacity and parallax.
- **Camera2D System:** Calculates orthographic projection matrices based on camera position, zoom, and viewport dimensions.
- **Headless Frame Capture:** Supports rendering directly into offscreen textures for automated headless screenshot generation and playtest assertion testing.
