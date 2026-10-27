---
id: cookbook.health-damage
title: Health, Damage, and Invulnerability Frames
summary: Player health pool, damage reception, knockback impulses, and post-hit invulnerability blinking.
tags: ["cookbook", "health", "damage", "invulnerability", "combat"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["cookbook.checkpoints-respawn", "agents.game-feel-defaults"]
---

# Recipe: Health & Damage

## Goal
Manage player hit points, apply damage and knockback on hazard contact, and grant brief invulnerability frames (i-frames).

## Code

```typescript <!-- check -->
// scripts/health_system.ts
export const MAX_HEALTH = 3;
export const INVULN_DURATION = 1.0;

let currentHealth = MAX_HEALTH;
let invulnTimer = 0.0;
let isDead = false;

export function takeDamage(amount: number, knockbackDir: number): void {
  if (invulnTimer > 0.0 || isDead) return;

  currentHealth -= amount;
  invulnTimer = INVULN_DURATION;
  forge.audio.play("hurt.wav", { bus: "sfx", volume: 0.9 });
  forge.log.info("Player took damage! Remaining health: " + currentHealth);

  if (currentHealth <= 0) {
    currentHealth = 0;
    isDead = true;
    forge.log.info("Player defeated.");
  }

  // Knockback application
  const transform = forge.entity.getTransform();
  transform.position[0] += knockbackDir * 24.0;
  transform.position[1] += 16.0;
  forge.entity.setTransform(transform);
}

export function onUpdate(dt: number): void {
  const safeDt = Math.min(dt, 0.05);

  if (invulnTimer > 0.0) {
    invulnTimer = Math.max(0.0, invulnTimer - safeDt);
  }
}
```

## How to Verify
1. Call `takeDamage(1, -1.0)` twice in rapid succession.
2. Confirm health only drops by 1 due to active `invulnTimer`.

## Variations
- **Sprite Flash:** Toggle sprite opacity or tint color during `invulnTimer > 0` to visually communicate i-frames.
- **Armor / Shields:** Consume shield points before deducting core health.
