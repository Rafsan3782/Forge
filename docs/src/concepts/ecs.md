---
id: concepts.ecs
title: Entity Component System Architecture
summary: Overview of Forge's high-performance bevy_ecs data-oriented foundation.
tags: ["concepts", "ecs", "architecture", "entities", "components"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.mental-model", "concepts.scenes", "concepts.scripting"]
---

# Entity Component System (ECS)

Forge is built on a data-oriented Entity-Component-System (ECS) architecture powered by `bevy_ecs`.

## Core Principles

- **Entities:** Simple numerical IDs (`Entity`) acting as containers for components.
- **Components:** Pure data structs stored in contiguous archetype tables (e.g. `Transform2D`, `Sprite`, `RigidBody2D`).
- **Systems:** Functions that execute queries over entities matching specific component tuples every frame (e.g. `camera_follow_system`, `sprite_render_system`).

## Component Hierarchy & Serialization

Entities maintain hierarchy through `Parent` and `Children` relationships:

- `Parent(Entity)`: Points to the parent entity.
- `Children(Vec<Entity>)`: Maintains ordered children.
- Transforms propagate down the hierarchy: parent position and rotation transform child local coordinates into global coordinates during the render stage.

All ECS components in Forge are registered with serde for serialization into `scene.json`.
