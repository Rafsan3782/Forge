---
id: concepts.audio
title: Audio Bus System and Spatialization
summary: Multi-channel audio mixer architecture powered by Rodio, bus controls, and sound sources.
tags: ["concepts", "audio", "sound", "music", "rodio"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.conventions", "agents.playbooks"]
---

# Audio System

Forge uses `rodio` for real-time audio playback, channel routing, and spatial volume attenuation.

## Mixer Architecture

1. **Audio Busses:**
   - `master`: Top-level volume affecting all audio.
   - `music`: Background tracks, looping soundtracks.
   - `sfx`: Gameplay sound effects, impacts, UI sounds.
2. **Components:**
   - `AudioSource`: Attached to entities with `asset_id`, `volume`, `pitch`, `loop`, and `autoplay`.
   - `AudioListener`: Defines ear position for 2D spatial attenuation (usually attached to Camera).
3. **Scripting API:**
   - `forge.audio.play(assetId, options)`: Play audio on designated bus.
   - `forge.audio.stop(playbackId, bus)`: Stop instance or entire bus.
   - `forge.audio.setBusVolume(bus, volume)`: Dynamically adjust gain (0.0 to 1.0).
