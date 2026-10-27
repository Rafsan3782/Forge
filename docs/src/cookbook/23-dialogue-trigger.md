---
id: cookbook.dialogue-trigger
title: Dialogue Triggers and Text Log Streaming
summary: Proximity-based NPC interaction, prompt displays, and dialogue step sequences.
tags: ["cookbook", "dialogue", "npc", "interaction", "logs"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["concepts.scripting", "agents.limits"]
---

# Recipe: Dialogue Triggers

## Goal
Trigger NPC dialogue when standing close and pressing the interact button, advancing conversation lines cleanly.

## Code

```typescript <!-- check -->
// scripts/dialogue.ts
const NPC_POS = [500.0, 80.0];
const INTERACT_RADIUS = 40.0;

const lines = [
  "Elder: Greetings, traveler!",
  "Elder: Beware the red patrollers beyond the bridge.",
  "Elder: Collect 5 coins to unlock the golden gate."
];

let lineIndex = 0;
let inDialogue = false;

export function onUpdate(_dt: number): void {
  const heroTransform = forge.entity.getTransform();
  const dx = heroTransform.position[0] - NPC_POS[0];
  const dy = heroTransform.position[1] - NPC_POS[1];
  const isNearby = (dx * dx + dy * dy) < (INTERACT_RADIUS * INTERACT_RADIUS);

  if (isNearby && forge.input.isActionJustPressed("interact")) {
    if (!inDialogue) {
      inDialogue = true;
      lineIndex = 0;
    } else {
      lineIndex += 1;
      if (lineIndex >= lines.length) {
        inDialogue = false;
        forge.log.info("Dialogue finished.");
        return;
      }
    }
    forge.log.info(lines[lineIndex]);
    forge.audio.play("talk.wav", { bus: "sfx", volume: 0.5 });
  }
}
```

## How to Verify
1. Stand at `[500.0, 80.0]`.
2. Press `"interact"` 3 times -> verify each dialogue line is logged in sequence.
3. 4th press ends dialogue.

## Variations
- **Branching Choices:** Select line based on whether `hasKey` is true.
- **Typing SFX:** Play small blip sound for each word in the dialogue string.
