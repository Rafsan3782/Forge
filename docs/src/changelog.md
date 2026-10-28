---
id: changelog
title: "Forge Engine Changelog"
summary: "Chronological release history and major milestone records for the Forge game engine."
tags: ["changelog", "releases", "history"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["guide-getting-started", "concept-architecture"]
---

# Forge Engine Changelog

## [0.1.0] - 2026-10-27

### Added
- **Engine Core:** High-performance 2D renderer using wgpu, sprite batching, and orthographic camera system.
- **Physics:** Rapier 2D physics integration with dynamic/kinematic bodies, colliders, and raycasting.
- **Audio Subsystem:** Multi-bus sound mixing (`master`, `music`, `sfx`) powered by Rodio.
- **Scripting Runtime:** QuickJS embedded engine with TypeScript SWC compilation and `forge.d.ts` declaration bindings.
- **Studio IDE:** Tauri 2 desktop host featuring React 19 Scene Tree, Inspector, Asset Browser, and Viewport panels.
- **IPC Protocol:** Bidirectional JSON-RPC 2.0 and MCP bridge on WebSocket port 9002.
- **Documentation System:** Unified multi-target docs generator emitting website bundles, agent packs, `llms.txt`, and machine-readable index.
- **Templates:** Hello World, Tilemap Demo, Script Demo, Physics Demo, Audio Demo, and Platformer starter games.
