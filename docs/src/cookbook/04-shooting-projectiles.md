---
id: cookbook.shooting-projectiles
title: Shooting Projectiles and Bullet Lifecycles
summary: Spawn, direction calculation, travel velocity, lifespan expiration, and cleanup of bullets.
tags: ["cookbook", "shooting", "combat", "projectiles", "spawn"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["cookbook.enemy-patrol-chase", "concepts.scripting"]
---

# Recipe: Shooting Projectiles

## Goal
Spawn moving projectile entities when firing, update their trajectories, and destroy them after expiration or impact.

## Code

```typescript <!-- check -->
// scripts/shooting.ts
interface Bullet {
  id: string;
  x: number;
  y: number;
  vx: number;
  vy: number;
  lifetime: number;
}

const BULLET_SPEED = 400.0;
const BULLET_LIFESPAN = 2.0;
const activeBullets: Bullet[] = [];

export function onUpdate(dt: number): void {
  const safeDt = Math.min(dt, 0.05);

  if (forge.input.isActionJustPressed("fire")) {
    const heroTransform = forge.entity.getTransform();
    const spawnX = heroTransform.position[0] + 16.0;
    const spawnY = heroTransform.position[1];

    const entityId = forge.spawn("Bullet", {
      Transform2D: { position: [spawnX, spawnY], rotation: 0, scale: [1, 1] },
      Sprite: { asset_id: "bullet.png", z_index: 15 }
    });

    activeBullets.push({
      id: entityId,
      x: spawnX,
      y: spawnY,
      vx: BULLET_SPEED,
      vy: 0.0,
      lifetime: BULLET_LIFESPAN
    });

    forge.audio.play("shoot.wav", { bus: "sfx", volume: 0.7 });
  }

  for (let i = activeBullets.length - 1; i >= 0; i--) {
    const b = activeBullets[i];
    b.lifetime -= safeDt;
    b.x += b.vx * safeDt;
    b.y += b.vy * safeDt;

    if (b.lifetime <= 0.0) {
      forge.destroy(b.id);
      activeBullets.splice(i, 1);
    }
  }
}
```

## How to Verify
1. Press `"fire"` action.
2. Confirm new bullet entity spawned in scene tree.
3. Confirm bullet entity is destroyed after exactly 2.0 seconds.

## Variations
- **Angled Spread Shot:** Spawn 3 bullets with velocity angles `-15°`, `0°`, and `+15°`.
- **Homing Missile:** Gradually rotate velocity vector towards closest enemy position.
