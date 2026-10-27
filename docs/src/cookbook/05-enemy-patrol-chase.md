---
id: cookbook.enemy-patrol-chase
title: Enemy AI Patrol and Target Chase States
summary: Two-state enemy AI switching between horizontal boundary patrol and aggro pursuit.
tags: ["cookbook", "enemy", "ai", "patrol", "chase", "state-machine"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["cookbook.health-damage", "agents.game-feel-defaults"]
---

# Recipe: Enemy Patrol & Chase

## Goal
Create an enemy AI that patrols horizontally between bounds and transitions into a chase state when the player enters detection range.

## Code

```typescript <!-- check -->
// scripts/enemy_ai.ts
type EnemyState = "patrol" | "chase";

export const PATROL_SPEED = 75.0;
export const CHASE_SPEED = 120.0;
export const AGGRO_RADIUS = 160.0;
export const LEFT_BOUND = 700.0;
export const RIGHT_BOUND = 900.0;

let state: EnemyState = "patrol";
let posX = 800.0;
let posY = 80.0;
let direction = 1.0;
const playerCoordX = 820.0; // Tracked via level coordinator or script property

export function onUpdate(dt: number): void {
  const safeDt = Math.min(dt, 0.05);
  const distToPlayer = Math.abs(posX - playerCoordX);

  if (distToPlayer < AGGRO_RADIUS) {
    state = "chase";
  } else {
    state = "patrol";
  }

  if (state === "patrol") {
    posX += direction * PATROL_SPEED * safeDt;
    if (posX >= RIGHT_BOUND) {
      posX = RIGHT_BOUND;
      direction = -1.0;
    } else if (posX <= LEFT_BOUND) {
      posX = LEFT_BOUND;
      direction = 1.0;
    }
  } else if (state === "chase") {
    direction = playerCoordX > posX ? 1.0 : -1.0;
    posX += direction * CHASE_SPEED * safeDt;
  }

  const transform = forge.entity.getTransform();
  transform.position[0] = posX;
  transform.position[1] = posY;
  forge.entity.setTransform(transform);
}
```

## How to Verify
1. Place player outside `AGGRO_RADIUS` -> observe enemy bouncing between `700` and `900`.
2. Move player within `160` px -> verify enemy switches direction and accelerates towards player.

## Variations
- **De-aggro Delay:** Keep enemy in chase state for 1.5 seconds after player leaves radius.
- **Leash Limit:** Enemy abandons pursuit if player lures it too far from original spawn point.
