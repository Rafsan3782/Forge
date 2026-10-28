---
id: guide-scripting-basics
title: "Scripting Basics in Forge"
summary: "Learn how to write TypeScript game scripts with lifecycle hooks and the global forge API."
tags: ["guide", "scripting", "basics"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["ref-script-index", "cookbook-player-controller", "ref-comp-script"]
---

# Scripting Basics in Forge

Gameplay logic in Forge is authored in TypeScript, transpiled via SWC, and executed inside QuickJS on the engine thread.

---

## Script Lifecycle

Scripts attached to an entity via the `Script` component export lifecycle callbacks:

```snippet
export function onStart(): void {
  const name = forge.entity.getName();
  forge.log.info("Entity started: " + name);
}

export function onUpdate(dt: number): void {
  const transform = forge.entity.getTransform();
  if (forge.input.isActionPressed("move_right")) {
    transform.position[0] += 100 * dt;
    forge.entity.setTransform(transform);
  }
}
```

---

## Key Lifecycle Callbacks

- `onStart()`: Called once when the entity enters play mode.
- `onUpdate(dt: number)`: Called every frame before rendering with frame delta time in seconds.
- `onFixedUpdate(dt: number)`: Called at constant 60Hz intervals for physics calculations.
- `onDestroy()`: Called when the entity is deleted or play mode ends.

---

## State Transfer During Hot Reload

Forge supports state preservation across script reloads via `serialize` and `deserialize`:

```snippet
let score: number = 0;

export function serialize(): unknown {
  return { score };
}

export function deserialize(state: unknown): void {
  if (typeof state === "object" && state !== null && "score" in state) {
    score = (state as { score: number }).score;
  }
}
```
