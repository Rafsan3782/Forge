---
id: cookbook.scene-transitions
title: Scene Transitions and Level Loading
summary: Triggering level completion, loading target scenes, and state handoff.
tags: ["cookbook", "scene", "transition", "level-load"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["concepts.ecs", "cookbook.saving-progress"]
---

# Recipe: Scene Transitions

## Goal
Detect level completion triggers and initiate scene loading into the next stage.

## Code

```typescript <!-- check -->
// scripts/scene_transition.ts
let transitioning = false;
let transitionTimer = 0.0;
const TRANSITION_DELAY = 1.0;
let nextSceneTarget = "";

export function triggerNextLevel(scenePath: string): void {
  if (transitioning) return;
  transitioning = true;
  nextSceneTarget = scenePath;
  transitionTimer = TRANSITION_DELAY;
  forge.audio.play("win.wav", { bus: "sfx", volume: 0.9 });
  forge.log.info("Level Complete! Transitioning to: " + scenePath);
}

export function onUpdate(dt: number): void {
  if (!transitioning) return;

  transitionTimer -= dt;
  if (transitionTimer <= 0.0) {
    transitioning = false;
    forge.scene.load(nextSceneTarget);
  }
}
```

## How to Verify
1. Call `triggerNextLevel("scenes/level_2.json")`.
2. Observe 1.0 second delay with win audio playing.
3. Verify `forge.scene.load` is called with target path.

## Variations
- **Fade to Black:** Scale down camera clear color brightness during delay.
- **Victory Pose:** Lock player input and play celebratory sprite animation during delay.
