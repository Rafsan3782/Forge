---
id: agents.mental-model
title: Engine Mental Model
summary: Fundamental architecture, data model, lifecycle loops, and execution modes of Forge.
tags: ["agents", "architecture", "ecs", "lifecycle"]
audience: agent
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.conventions", "agents.rules-and-invariants", "concepts.ecs", "concepts.scripting"]
---

# Engine Mental Model

Forge is an ECS-first 2D game engine built in Rust (`bevy_ecs`, `wgpu`, `rapier2d`, `rodio`) with an embedded QuickJS (`rquickjs`) TypeScript scripting runtime and a JSON-RPC / MCP WebSocket bridge.

## 1. Engine Core Concepts

| Primitive | Definition | Storage / Persistence |
| :--- | :--- | :--- |
| **Entity** | Lightweight 64-bit integer ID identifying a game object. Contains no data or logic on its own. | Spawned in `bevy_ecs::World`. Identified stably across saves by `EntityId(String)`. |
| **Component** | Plain data struct attached to an entity (e.g. `Transform2D`, `Sprite`, `RigidBody2D`). | Dense archetype tables in ECS. Serialized under entity records in `scene.json`. |
| **Scene** | Flat list of root entities with hierarchy references and serialized components. | JSON document (`scene.json`) parsed and saved via `SceneSerializer`. |
| **Asset** | File in `assets/` tracked by UUID with sidecar metadata (`.meta.json`). | Binary files (`.png`, `.wav`, `.spritesheet.json`, `.json`) loaded via `forge_assets`. |
| **Script** | TypeScript module executed inside embedded QuickJS. Attached to an entity via `Script` component. | Files in `scripts/` (e.g. `scripts/player.ts`). Transpiled to JS via SWC at load time. |

## 2. Frame Execution Order

Every simulation frame executes strictly in sequence:

```text
1. Engine Poll (Window events, Win32 message loop)
2. IPC Ingestion (WebSocket JSON-RPC / MCP messages from bridge)
3. Input Update (Keyboard, mouse, gamepad, simulated input queues)
4. Fixed Update Loop (Accumulated dt >= fixed_timestep, default 60 Hz / 16.666 ms):
   a. Script onFixedUpdate(fixed_dt)
   b. Rapier 2D Physics Step (Rigid bodies, colliders, velocity resolution)
   c. Physics Contact & Trigger Detection
5. Variable Update:
   a. Asset Hot Reload checks
   b. Script onStart() (first frame for newly spawned scripts)
   c. Script onUpdate(variable_dt)
   d. Script Command Queue Flushing (spawn, destroy, setComponent, setTransform)
   e. Transform Hierarchy Propagation (Parent matrix -> Children global positions)
   f. Animation Systems (Frame advancement, UV offset updates)
   g. Audio Playback Queue Flushing (Sound triggers -> Rodio buses)
6. Render Stage:
   a. Camera Bounds Calculation
   b. WGPU RenderPass (Clear background -> Tilemap layers -> Sprite batch -> Gizmos)
   c. Swapchain Present or Headless Framebuffer Capture
```

## 3. Edit Mode versus Play Mode

| Property | Edit Mode | Play Mode |
| :--- | :--- | :--- |
| **Simulation** | Stopped. Fixed and variable delta times are 0. | Running at 60 FPS fixed physics, variable render. |
| **Scene State** | Authoring state. Saved to disk on `scene.save`. | Transient runtime state. Restored to snapshot on `play.stop`. |
| **Scripts** | Do not execute (`onStart`, `onUpdate` are inactive). | All attached scripts run their lifecycle hooks. |
| **Physics** | Colliders visible as editor wireframes; no velocity or gravity. | Active Rapier simulation with gravity and collision response. |
| **Camera** | Editor viewport camera controllable via mouse pan/zoom. | Game camera (`Camera2D` entity with highest priority) controls view. |
| **Hot Reload** | Asset and scene updates apply immediately. | Script changes can hot reload with state preserved via `serialize()` / `deserialize()`. |
