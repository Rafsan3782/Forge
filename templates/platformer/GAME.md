# Platformer

## Concept
<!-- id: concept -->
A fast-paced 2D retro action platformer game built on the Forge engine. Players control a hero traversing floating platforms, timing precise jumps, gathering shiny golden coins, dodging static spike hazards and roaming patroller enemies, to reach the level exit flag.

## Pillars
<!-- id: pillars -->
- **Tight Kinematic Controls**: Responsive physics with variable jump height, coyote time, and jump buffering.
- **Rewarding Mastery**: Clear hazard telegraphing and fluid movement pacing that rewards skillful execution.
- **Audio-Visual Harmony**: Cohesive pixel art aesthetic paired with nostalgic 8-bit sound effects and chiptune soundtrack.

## Core Loop
<!-- id: core-loop -->
1. **Traverse**: Navigate horizontal terrain and floating platforms using run and jump controls.
2. **Collect**: Gather optional coins scattered throughout high-risk routes for score and completion.
3. **Survive**: Avoid lethal spikes and enemy patrollers; respawn at the latest reached checkpoint upon defeat.
4. **Achieve**: Reach the victory flag pole at the level boundary to trigger level completion.

## Controls
<!-- id: controls -->
- **Move Left / Right**: `A` / `D` or `ArrowLeft` / `ArrowRight` (Gamepad Left Stick X / D-Pad)
- **Jump**: `Space`, `W`, or `ArrowUp` (Gamepad South Button `A`)
- **Variable Jump**: Release jump early to cut upward velocity and perform short hops.

## Art Direction
<!-- id: art-direction -->
- **Style**: Vibrant 16x16 / 32x32 retro pixel art sprites and tilemaps.
- **Palette**: High contrast outdoor meadow with lush green terrain, blue skies, stone ledges, and glowing golden collectibles.
- **Animation**: Distinct multi-frame character runs, jump arcs, patroller patrol cycles, and animated spinning coins.

## Audio Direction
<!-- id: audio-direction -->
- **SFX**: Snappy, energetic procedural retro sound effects for jumping, collecting coins, and taking damage.
- **BGM**: Uplifting melodic chiptune music looping on the `music` audio bus at balanced volume.
- **Buses**: Separate audio routing for `master`, `music`, and `sfx`.

## Scope Limits
<!-- id: scope-limits -->
- **Single-Screen to Linear Scrolling**: Single horizontal linear stage clamped within 1536x600 px boundaries.
- **Kinematic Physics**: Hand-tuned raycast / AABB platform collisions rather than heavy dynamic rigidbodies.
- **No Network Multiplayer**: Purely single-player local platforming experience.
