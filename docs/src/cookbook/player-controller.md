---
id: cookbook-player-controller
title: "2D Platformer Player Controller"
summary: "Step-by-step recipe for building a responsive 2D platformer character controller with jumping and raycast ground checks."
tags: ["cookbook", "recipe", "character", "physics"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["guide-scripting-basics", "ref-comp-rigidbody2d", "ref-comp-collider2d"]
---

# 2D Platformer Player Controller

This recipe demonstrates how to create a 2D platformer character with horizontal velocity movement, variable jump height, and audio trigger integration.

---

## 1. Scene Setup

In `scene.json`, attach `RigidBody2D`, `Collider2D`, `Sprite`, and `Script` to your player entity:

```json
{
  "name": "Player",
  "components": {
    "Transform2D": { "position": [100.0, 150.0], "rotation": 0.0, "scale": [1.0, 1.0] },
    "RigidBody2D": { "body_type": "dynamic", "gravity_scale": 1.5, "lock_rotation": true },
    "Collider2D": { "shape": "box", "size": [16.0, 24.0], "friction": 0.1 },
    "Script": { "path": "scripts/player.ts", "enabled": true }
  }
}
```

---

## 2. Controller Script (`scripts/player.ts`)

```snippet
const MOVE_SPEED = 200.0;
const JUMP_FORCE = 320.0;

let isGrounded = false;

export function onUpdate(dt: number): void {
  const transform = forge.entity.getTransform();
  let moveX = 0;

  if (forge.input.isActionPressed("move_left")) {
    moveX -= 1;
  }
  if (forge.input.isActionPressed("move_right")) {
    moveX += 1;
  }

  // Check ground contact via raycast
  const rayOrigin: [number, number] = [transform.position[0], transform.position[1] - 12];
  const hit = forge.physics.raycast(rayOrigin, [0, -1], 4.0);
  isGrounded = hit !== null;

  // Jump trigger
  if (isGrounded && forge.input.isActionJustPressed("jump")) {
    transform.position[1] += JUMP_FORCE * dt;
    forge.audio.play("assets/jump.wav", { bus: "sfx", volume: 0.7 });
  }

  transform.position[0] += moveX * MOVE_SPEED * dt;
  forge.entity.setTransform(transform);
}
```

---

## Common Mistakes & Invariants

- **Always lock rotation:** Ensure `lock_rotation` is enabled on platformer player bodies to prevent physics tipping.
- **Normalize input axes:** When checking 2D diagonals, normalize the vector to avoid faster diagonal speed.
