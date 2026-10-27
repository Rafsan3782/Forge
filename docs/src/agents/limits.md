---
id: agents.limits
title: Engine Limitations and Unsupported Features
summary: Explicit inventory of features not supported in the current engine, preventing AI hallucination.
tags: ["agents", "limits", "unsupported", "constraints"]
audience: agent
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.mental-model", "agents.rules-and-invariants"]
---

# Engine Limitations & Unsupported Features

AI agents must NEVER invent, import, or assume support for the following capabilities in the Phase 1/Phase 2 engine:

## 1. Explicitly Unsupported Systems

| Feature Area | Unsupported Capability | Reason / Current Architecture | Valid Workaround |
| :--- | :--- | :--- | :--- |
| **UI & Text** | In-engine vector font rendering, `Text2D` components, HTML/CSS canvas in-game overlay. | `forge_render` does not have an SDF or glyph rasterization pipeline yet. | Use sprite icons anchored to camera, digit sprite sheets, or `forge.log.info()` streaming. |
| **Scripting Queries** | `forge.entity.getTransformOf(id)` or `forge.scene.findEntity(name)`. | QuickJS bindings only bind the executing entity's transform for isolation. | Centralize game state coordinates in a level coordinator script or pass via shared constants. |
| **Physics Callbacks** | Automatic execution of `onCollisionEnter` or `onTriggerEnter` hooks in TypeScript. | Rapier contact event buffers are not dispatched into QuickJS callbacks. | Perform explicit AABB bounding box or circle distance checks in `onUpdate(dt)`. |
| **Script Raycasting** | `forge.physics.raycast` returning valid hits. | Stubbed to return `None` in `forge_script`. | Kinematic AABB mathematics against known platform geometry arrays. |
| **Audio Formats** | MP3, OGG, FLAC streaming playback. | Rodio audio pipeline currently only initializes 16-bit uncompressed WAV decoding. | Pre-generate or transcode audio assets to 16-bit PCM `.wav`. |
| **3D Rendering** | 3D meshes, perspective cameras, PBR materials, depth buffers. | Forge is strictly a 2D engine (`wgpu` sprite batching & tilemaps). | Use 2D sprites, layered parallax, or isometric tilemaps. |
| **Multiplayer / Net** | Built-in networked physics synchronization or multiplayer lobbies. | No replication protocol in engine runtime. | Local single-player simulation only. |
| **Physics Controller** | Built-in `CharacterController2D` script bindings. | Component exists in Rust crate but is not serialized or exposed to QuickJS. | Kinematic TypeScript character controller (see `templates/platformer/scripts/player.ts`). |
| **Sprite Animation Sync** | `forge.animation.play(clip)` changing `Sprite.asset_id` automatically. | Animator command queue does not rebind texture asset ID during frame advance. | Swap texture IDs explicitly or use procedural sprite sheets with configured UV frames. |
