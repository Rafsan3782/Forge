---
id: concepts.input
title: Input Mapping and Polling
summary: Unified keyboard, mouse, and gamepad action mapping architecture with synthetic input injection.
tags: ["concepts", "input", "controls", "actions", "simulation"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.conventions", "agents.playbooks"]
---

# Input Mapping and Polling

Forge decouples physical hardware devices from game logic through an action-mapping layer configured via `input_map.json`.

## Mapping Architecture

- **Action Bindings:** Abstract action names (e.g. `"jump"`, `"move_left"`, `"fire"`) map to one or more physical inputs (keys, mouse buttons, gamepad buttons).
- **Polling Methods:**
  - `forge.input.isActionPressed(action)`: True while input is held down.
  - `forge.input.isActionJustPressed(action)`: True on the exact frame the action was activated.
  - `forge.input.isActionJustReleased(action)`: True on the exact frame the action was released.
  - `forge.input.getAxis(axis)`: Returns continuous float between `-1.0` and `1.0`.
  - `forge.input.getMousePosition()`: Returns current cursor `[x, y]` in screen pixels.
- **Simulated Input:** `game.simulate_input` bridge tool injects synthetic keyboard or action events for automated headless testing.
