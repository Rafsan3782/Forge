---
id: concepts.scripting
title: QuickJS TypeScript Scripting Runtime
summary: Embedded QuickJS virtual machine, TypeScript transpilation, and global host API bindings.
tags: ["concepts", "scripting", "quickjs", "typescript", "forge-api"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.mental-model", "agents.limits"]
---

# QuickJS Scripting Runtime

Forge embeds QuickJS (`rquickjs`) to deliver zero-overhead, memory-safe TypeScript game scripting.

## Script Architecture

- **Transpiler:** Scripts authored in TypeScript are transpiled to JavaScript on load using embedded SWC.
- **Global `forge` Object:** Global runtime interface providing host functions:
  - `forge.entity`: Query ID, name, children, and host transform.
  - `forge.input`: Poll actions (`isActionPressed`), buttons, and axes.
  - `forge.time`: Delta time, frame count, elapsed duration.
  - `forge.audio`: Trigger audio playback across buses (`master`, `music`, `sfx`).
  - `forge.tilemap`: Get and set dynamic tile values.
  - `forge.spawn` / `forge.destroy`: Entity lifecycle manipulation.
  - `forge.log`: Structured log streaming to Studio console and telemetry.
- **Lifecycle Callbacks:** Optional exported functions: `onStart()`, `onUpdate(dt)`, `onFixedUpdate(dt)`, `onDestroy()`, `serialize()`, `deserialize(state)`.
