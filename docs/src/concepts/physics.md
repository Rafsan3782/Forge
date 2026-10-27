---
id: concepts.physics
title: 2D Physics and Kinematics
summary: Rapier 2D physics integration, rigid body simulation, colliders, and kinematic movement.
tags: ["concepts", "physics", "rapier", "collision", "kinematics"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.conventions", "agents.game-feel-defaults"]
---

# 2D Physics & Kinematics

Forge integrates the Rapier 2D physics engine for realistic rigid body mechanics, while supporting direct kinematic character control.

## Subsystem Architecture

- **PhysicsWorld2D:** Hosts Rapier `RigidBodySet`, `ColliderSet`, gravity vectors, and contact event handlers.
- **Body Types:**
  - `Dynamic`: Full physics response driven by forces, impulses, and gravity.
  - `KinematicPositionBased`: Driven directly by script transforms; ignores gravity and forces but pushes dynamic bodies.
  - `KinematicVelocityBased`: Moves according to velocity vector.
  - `Fixed`: Static obstacles, floors, and walls with infinite mass.
- **Collider Shapes:** `Box(half_extents)`, `Circle(radius)`, `Capsule`, and `Segment`.
- **Sensor Colliders:** Colliders flagged as `sensor: true` pass through objects while registering overlap volumes.
