---
id: cookbook.audio-cues
title: Audio Cues and Dynamic Bus Volume Control
summary: Managing background music loops, pitch-randomized sound effects, and bus volume fading.
tags: ["cookbook", "audio", "sfx", "music", "volume"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["concepts.audio", "agents.game-feel-defaults"]
---

# Recipe: Audio Cues & Bus Controls

## Goal
Play loopable background music and pitch-randomized sound effects while managing volume attenuation per bus.

## Code

```typescript <!-- check -->
// scripts/audio_manager.ts
let musicPlaying = false;

export function initAudio(): void {
  forge.audio.setBusVolume("master", 1.0);
  forge.audio.setBusVolume("music", 0.6);
  forge.audio.setBusVolume("sfx", 0.8);
}

export function startMusic(): void {
  if (musicPlaying) return;
  forge.audio.play("music.wav", { bus: "music", loop: true, volume: 0.6 });
  musicPlaying = true;
}

export function playSfx(soundAsset: string): void {
  // Minor pitch randomization for variation
  const pitch = 0.95 + Math.random() * 0.1;
  forge.audio.play(soundAsset, { bus: "sfx", volume: 0.8, pitch });
}

export function muteSfx(mute: boolean): void {
  forge.audio.setBusVolume("sfx", mute ? 0.0 : 0.8);
}
```

## How to Verify
1. Call `initAudio()` and `startMusic()`.
2. Inspect log output confirming `"music.wav"` started on bus `"music"`.
3. Call `muteSfx(true)` -> confirm `"sfx"` bus volume set to `0.0`.

## Variations
- **Duck Music on Dialogue:** Lower music bus volume to 0.2 during voice playback.
- **Low Health Warning:** Rapidly loop an alert beep on sfx bus when health reaches 1.
