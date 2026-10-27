---
id: cookbook.state-machines
title: Explicit State Machines for Gameplay Logic
summary: Structured state machine pattern with enter, update, and exit state transitions.
tags: ["cookbook", "state-machine", "architecture", "fsm"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["cookbook.enemy-patrol-chase", "agents.mental-model"]
---

# Recipe: Finite State Machines (FSM)

## Goal
Implement a clean, bug-free finite state machine for handling complex player or enemy states.

## Code

```typescript <!-- check -->
// scripts/state_machine.ts
export type StateId = "idle" | "run" | "jump" | "fall";

let currentState: StateId = "idle";
let stateTime = 0.0;

export function changeState(nextState: StateId): void {
  if (currentState === nextState) return;

  // Exit current state
  forge.log.info(`Exiting ${currentState} after ${stateTime.toFixed(2)}s`);

  currentState = nextState;
  stateTime = 0.0;

  // Enter next state
  if (currentState === "jump") {
    forge.audio.play("jump.wav", { bus: "sfx" });
  }
}

export function onUpdate(dt: number): void {
  const safeDt = Math.min(dt, 0.05);
  stateTime += safeDt;

  const isMoving = forge.input.isActionPressed("move_left") || forge.input.isActionPressed("move_right");
  const isGrounded = true; // Bound to physics check

  switch (currentState) {
    case "idle":
      if (forge.input.isActionJustPressed("jump")) changeState("jump");
      else if (isMoving) changeState("run");
      break;

    case "run":
      if (forge.input.isActionJustPressed("jump")) changeState("jump");
      else if (!isMoving) changeState("idle");
      break;

    case "jump":
      if (stateTime > 0.4) changeState("fall");
      break;

    case "fall":
      if (isGrounded) changeState(isMoving ? "run" : "idle");
      break;
  }
}
```

## How to Verify
1. Start in `"idle"` -> press right arrow -> observe state transitioning to `"run"`.
2. Press jump -> observe transition to `"jump"` and audio trigger.

## Variations
- **Action State Locking:** Block direction changes while in a locked state (e.g. `"attack"`).
- **Sub-states:** Nest combat states within grounded movement states.
