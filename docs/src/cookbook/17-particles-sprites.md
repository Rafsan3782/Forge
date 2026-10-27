---
id: cookbook.particles-sprites
title: Particle Effects Using Lightweight Sprites
summary: Spawning burst particle bursts (sparks, dust) using pooled or timed sprite entities.
tags: ["cookbook", "particles", "effects", "sprites", "burst"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["concepts.rendering", "cookbook.health-damage"]
---

# Recipe: Sprite Particle Bursts

## Goal
Spawn a burst of small particle sprites on jump, impact, or death that scatter and fade out.

## Code

```typescript <!-- check -->
// scripts/particle_burst.ts
interface Particle {
  id: string;
  x: number;
  y: number;
  vx: number;
  vy: number;
  life: number;
}

const particles: Particle[] = [];

export function emitBurst(originX: number, originY: number, count: number): void {
  for (let i = 0; i < count; i++) {
    const angle = Math.random() * Math.PI * 2.0;
    const speed = 50.0 + Math.random() * 100.0;
    const vx = Math.cos(angle) * speed;
    const vy = Math.sin(angle) * speed;

    const id = forge.spawn(`P_${Date.now()}_${i}`, {
      Transform2D: { position: [originX, originY], rotation: 0, scale: [0.5, 0.5] },
      Sprite: { asset_id: "spark.png", z_index: 20 }
    });

    particles.push({ id, x: originX, y: originY, vx, vy, life: 0.5 });
  }
}

export function updateParticles(dt: number): void {
  for (let i = particles.length - 1; i >= 0; i--) {
    const p = particles[i];
    p.life -= dt;
    p.x += p.vx * dt;
    p.y += p.vy * dt;
    p.vy -= 400.0 * dt; // Gravity

    if (p.life <= 0.0) {
      forge.destroy(p.id);
      particles.splice(i, 1);
    }
  }
}
```

## How to Verify
1. Call `emitBurst(100, 100, 8)` -> confirm 8 entities spawn.
2. Step 0.6 seconds forward -> confirm all 8 entities are destroyed.

## Variations
- **Smoke Puff:** Particles rise upward with low speed and expand in scale.
- **Color Fading:** Reduce alpha tint from `1.0` down to `0.0` over particle lifetime.
