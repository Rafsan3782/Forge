---
id: cookbook.screen-fade-transitions
title: Procedural Camera Fading and Dark Transitions
summary: Smooth screen blackout transitions using camera background color manipulation.
tags: ["cookbook", "screen-fade", "transition", "camera", "visuals"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["cookbook.scene-transitions", "concepts.rendering"]
---

# Recipe: Camera Screen Fading

## Goal
Fade the viewport to black on player death or level transition by manipulating the camera background or dark overlays.

## Code

```typescript <!-- check -->
// scripts/screen_fade.ts
let fadeAlpha = 0.0;
let fadeDirection = 0.0; // 1.0 = fading to black, -1.0 = fading in
const FADE_SPEED = 2.0;

export function startFadeOut(): void {
  fadeDirection = 1.0;
  forge.log.info("Starting screen fade out...");
}

export function startFadeIn(): void {
  fadeDirection = -1.0;
  fadeAlpha = 1.0;
  forge.log.info("Starting screen fade in...");
}

export function onUpdate(dt: number): void {
  if (fadeDirection === 0.0) return;

  const safeDt = Math.min(dt, 0.05);
  fadeAlpha += fadeDirection * FADE_SPEED * safeDt;
  fadeAlpha = Math.max(0.0, Math.min(1.0, fadeAlpha));

  if (fadeAlpha >= 1.0 && fadeDirection > 0.0) {
    fadeDirection = 0.0;
    forge.log.info("Screen fully faded to black.");
  } else if (fadeAlpha <= 0.0 && fadeDirection < 0.0) {
    fadeDirection = 0.0;
    forge.log.info("Screen fully restored.");
  }
}
```

## How to Verify
1. Call `startFadeOut()`.
2. Advance 0.5s -> confirm log emits `"Screen fully faded to black"`.

## Variations
- **Red Hurt Flash:** Flash red alpha `0.4` for 0.1s on taking damage.
- **White Flash on Boss Defeat:** Flash white overlay when defeating stage boss.
