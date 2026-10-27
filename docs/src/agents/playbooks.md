---
id: agents.playbooks
title: Step-by-Step Agent Playbooks
summary: Deterministic procedures with verification criteria for common development workflows in Forge.
tags: ["agents", "playbooks", "workflows", "verification"]
audience: agent
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.the-work-loop", "agents.game-feel-defaults", "agents.limits"]
---

# Agent Playbooks

Follow these exact procedures when executing game development tasks.

---

## Playbook 1: Build a New Game from a Description

1. **Analyze Requirements:** Extract genre (platformer, top-down, puzzle), core mechanic, winning condition, and failure state.
2. **Setup Folder & Metadata:** Verify `assets/`, `scripts/`, `scene.json`, and `input_map.json`.
3. **Configure Input Actions:** Define action mappings (`move_left`, `move_right`, `jump`, `fire`) in `input_map.json`.
4. **Create Root Entities in `scene.json`:**
   - Add `Main Camera` entity with `Transform2D`, `Camera2D`, and tracking `Script`.
   - Add `World Tilemap` or ground geometry entities.
   - Add `Player` entity with `Transform2D`, `Sprite`, and gameplay `Script`.
5. **Implement Primary Controller Script:**
   - Write kinematic movement controller in `scripts/player.ts` using game-feel defaults.
   - Implement `onStart()` to log initialization and start background music.
   - Implement `onUpdate(dt)` with clamped delta time and input polling.
6. **Verify:**
   - Run `script.check` -> 0 diagnostic errors.
   - Run headless simulation for 10 frames -> verify no panic in `game.log`.
   - Capture `game.screenshot` -> verify camera frames player sprite clearly.

---

## Playbook 2: Add a Mechanic (e.g. Double Jump, Dash, or Shooting)

1. **State Definition:** Declare new state variables in host script (e.g. `let hasDoubleJump = true;` or `let dashTimer = 0.0;`).
2. **Input Mapping:** If new key required, add action mapping in `input_map.json`.
3. **Logic Integration:**
   - In `onUpdate(dt)`, check action trigger condition (`isActionJustPressed`).
   - Modify velocity or spawn projectile entity.
   - Play distinct sound effect via `forge.audio.play("sfx.wav", { bus: "sfx" })`.
4. **Verification:**
   - Run headless simulation with `game.simulate_input` invoking the new action.
   - Check `game.log` for execution logs (e.g. `"Double jump executed"`).

---

## Playbook 3: Build a Level with Tilemaps

1. **Palette Check:** Query `tileset.json` to verify tile IDs and texture references.
2. **Clear or Initialize Layer:** Use `tilemap.clear` or instantiate `Tilemap` component.
3. **Paint Level Geometry:**
   - Use `tilemap.fill_rect` for large solid ground spans and platforms.
   - Use `tilemap.set_tile` for decorative corners and specific obstacle tiles.
4. **Collision Registration:** Ensure platform bounding rects match painted tile coordinates.
5. **Verification:**
   - Call `tilemap.get` at known ground coordinates to verify tile ID.
   - Capture `game.screenshot` to visually inspect level structure.

---

## Playbook 4: Add Enemies (Patrollers & Chasers)

1. **Define Enemy Entity:** Create entity in `scene.json` with `Transform2D`, `Sprite` (tinted red or distinct sprite), and script.
2. **Implement AI Script:**
   - For patroller: move horizontally between bounds; flip direction on limit reach.
   - For chaser: compute direction vector towards player coordinates; step towards target.
3. **Player Hazard Overlap:** In coordinator or player script, check AABB distance against enemy position. Trigger player damage/respawn on touch.
4. **Verification:**
   - Run headless for 30 frames; log enemy coordinates over time to verify movement.

---

## Playbook 5: Add Sounds and Music

1. **Verify Asset:** Ensure uncompressed 16-bit PCM `.wav` exists in `assets/` with `.meta.json`.
2. **Background Music:** Call `forge.audio.play("music.wav", { bus: "music", loop: true, volume: 0.6 })` inside `onStart()`.
3. **Sound Effects:** Trigger sound calls (`jump.wav`, `coin.wav`, `hurt.wav`) in event response blocks inside `onUpdate(dt)`.
4. **Bus Volume:** Ensure buses (`master`, `music`, `sfx`) are unmuted.
5. **Verification:**
   - Inspect `game.log` for audio playback confirmations.

---

## Playbook 6: Fix a Bug from an Error Log

1. **Locate Error Source:** Extract file, line number, and error message from log.
2. **Cross-reference Troubleshooting Catalog:** Match symptom against `rules-and-invariants.md`.
3. **Inspect Active Code:** Use `script.read` to examine the failing lines.
4. **Apply Fix:** Write minimal drop-in correction via `script.write`.
5. **Verification:**
   - Re-run `script.check`.
   - Run headless for the frame count that previously failed.

---

## Playbook 7: Tune Game Feel

1. **Audit Parameters:** Identify the mechanic needing tuning (jump arc, friction, camera lag).
2. **Consult Tables:** Look up recommended values in `game-feel-defaults.md`.
3. **Update Script Constants:** Adjust constants in script header (`MOVE_SPEED`, `GRAVITY`, `COYOTE_TIME`).
4. **Simulate & Measure:**
   - Measure jump apex time and distance using automated input tests.
   - Verify landing feels responsive without sliding.

---

## Playbook 8: Make a Menu or HUD (with Current Engine Constraints)

1. **Acknowledge Limitation:** Phase 1 lacks in-engine vector font / text UI rendering.
2. **Adopt Supported Strategy:**
   - Option A: Render HUD icons (coins, hearts, keys) as sprite entities anchored relative to the camera position.
   - Option B: Use digit sprite frames (0-9) to display numbers.
   - Option C: Stream numeric metrics to Studio IDE and runner logs via `forge.log.info("HUD: Coins=5 Health=3")`.
3. **Never Attempt:** Do not attempt to attach non-existent UI components like `Text2D` or `Canvas` in `scene.json`.

---

## Playbook 9: Prepare a Release Build

1. **Verify Clean Workspace:** Run `cargo check --workspace` and `pnpm -r typecheck`.
2. **Asset Validation:** Ensure all assets referenced in `scene.json` and scripts exist in `assets/`.
3. **Scene Reset:** Verify `scene.json` is saved in initial state (player at starting spawn point, score at 0).
4. **Headless Smoke Test:** Run headless execution for 60 frames on the target scene. Confirm 0 error logs.
