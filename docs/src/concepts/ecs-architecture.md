---
id: concept-ecs-architecture
title: "Entity Component System (ECS) Design"
summary: "Understand how entities, components, and systems operate inside Forge scenes."
tags: ["concept", "ecs", "scene"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["ref-components-index", "concept-architecture", "ref-tool-scene-set_component"]
---

# Entity Component System (ECS) Design

Forge structures all game data using an Entity Component System pattern powered by `bevy_ecs`.

---

## Entities

An entity is a lightweight integer identifier representing a distinct game object in the world. Hierarchy links (`Parent` and `Children`) allow grouping entities into trees.

---

## Components

Components are pure data records attached to entities. Built-in components include:
- `Name`: Display label.
- `Transform2D`: 2D position, rotation, and scale.
- `Sprite`: Texture quad with tint and sorting order.
- `RigidBody2D`: Dynamic, kinematic, or static physics body.
- `Collider2D`: Bounding shape for collision.
- `Script`: Attached behavior code.
- `Tilemap`: Grid-based tile graphics.

---

## Systems

Systems run concurrently on the engine thread to process entities with matching component queries:
1. Input polling system updates `InputState`.
2. Physics step system advances Rapier simulation.
3. Script update system dispatches `onUpdate(dt)`.
4. Render system collects sprite and tilemap instances into GPU batches.
