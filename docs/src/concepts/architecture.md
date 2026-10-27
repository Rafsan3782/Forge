---
id: concept-architecture
title: "Engine Architecture & IPC Bridge"
summary: "High-level overview of the Forge engine core, Tauri Studio host, and WebSocket JSON-RPC bridge."
tags: ["concept", "architecture", "engine"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["concept-ecs-architecture", "ref-tools-index", "ref-cli-index"]
---

# Engine Architecture & IPC Bridge

Forge is architected across three decoupled boundaries:

```text
┌────────────────────────────────┐
│   Studio IDE (Tauri 2 + React) │
└───────────────┬────────────────┘
                │ JSON-RPC 2.0 / MCP (WebSocket port 9002)
┌───────────────▼────────────────┐
│  Rust Engine (wgpu + bevy_ecs) │
└───────────────┬────────────────┘
                │ Embedded VM (QuickJS)
┌───────────────▼────────────────┐
│   TypeScript Gameplay Scripts  │
└────────────────────────────────┘
```

---

## 1. Rust Engine Core

The engine loop manages:
- **`forge_core`**: ECS entity world, components, scene serialization, undo/redo stack.
- **`forge_render`**: wgpu pipeline batching textured quads, cameras, and tilemaps.
- **`forge_physics`**: Rapier 2D rigid bodies, colliders, contact events, and raycasting.
- **`forge_audio`**: Rodio-powered spatialized sound and multi-bus mixing.
- **`forge_script`**: QuickJS VM hosting TypeScript scripts with hot reloading.

---

## 2. IPC Protocol

The engine and IDE communicate via a lightweight, typed WebSocket protocol:
- Port 9002 default (overridable via `--port`).
- Bearer token authentication stored in `.forge/bridge.json`.
- Standard JSON-RPC 2.0 requests, responses, and streamed notifications.
- MCP (`Model Context Protocol`) compliant `tools/list` and `tools/call` endpoints.
