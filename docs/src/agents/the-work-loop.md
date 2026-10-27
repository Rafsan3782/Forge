---
id: agents.the-work-loop
title: The Agent Work Loop
summary: Deterministic cycle of reading, changing, checking, running headless, inspecting screenshots, and fixing.
tags: ["agents", "workflow", "work-loop", "headless", "verification"]
audience: agent
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.rules-and-invariants", "agents.playbooks"]
---

# The Agent Work Loop

Autonomous agents must execute a disciplined 6-step loop when building or modifying games in Forge:

```text
[1. Read] -> [2. Change] -> [3. Check] -> [4. Run Headless] -> [5. Screenshot] -> [6. Fix/Refine]
```

## Step-by-Step Cycle

### 1. Read (Understand Current State)
- Query the scene hierarchy via `scene.get_tree`.
- Check registered assets via `asset.list`.
- Read active scripts via `script.read` to understand existing state and variables.
- Inspect `input_map.json` to verify configured action names before binding inputs.

### 2. Change (Make Focused Modifications)
- Update or create scripts in `scripts/` using `script.write`.
- Add or modify entities and components using `scene.create_entity` or `scene.set_component`.
- Configure tilemaps using `tilemap.set_tile` or `tilemap.fill_rect`.
- Import textures or audio assets using `asset.import`.

### 3. Check (Compile & Static Verify)
- Verify script syntax and type correctness using `script.check` (or `forge check --project <path>`).
- If TypeScript diagnostics are returned, fix all errors before proceeding to simulation.
- Verify JSON syntax of `scene.json` and `input_map.json`.

### 4. Run Headless (Simulate Behavior)
- Launch headless simulation for N frames (e.g. 5 to 60 frames) using `forge_runtime --headless --frames 30 --scene <path>/scene.json`.
- Inject automated input using `game.simulate_input` (e.g. simulate `"jump"` or `"move_right"`).
- Capture simulation logs via `game.log` to confirm initialization (`onStart`) and updates (`onUpdate`).

### 5. Inspect Screenshot (Visual Sanity)
- Capture a viewport screenshot using `game.screenshot`.
- Check sprite visibility, camera framing, z-index ordering, and collision alignment.
- Verify sprites are not culled, flipped unexpectedly, or rendered outside the camera frustum.

### 6. Fix & Refine
- If errors occurred in logs, inspect the error stack trace and component state.
- Check against the Common Mistakes catalog in `rules-and-invariants.md`.
- Make targeted adjustments and repeat the cycle until all acceptance checks pass.

## Verification Matrix

| Change Type | Primary Verification Tool | What to Check |
| :--- | :--- | :--- |
| **Script Edit** | `script.check` + `game.log` | Zero TypeScript errors; no null reference crashes on `onStart`. |
| **Physics / Collision** | `physics.raycast` + headless run | Entity velocities resolve properly; player stops on platform tops. |
| **Camera Tuning** | `game.screenshot` | Player remains centered or within deadzone; bounds do not show void. |
| **Tilemap Painting** | `tilemap.get` + `game.screenshot` | Tile IDs correspond to correct palette tiles; chunk indices valid. |
| **Audio Setup** | `audio.play` test + log | WAV file loaded on target bus; volume within [0.0, 1.0]. |
