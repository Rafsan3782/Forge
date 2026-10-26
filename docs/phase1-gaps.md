# Phase 1 Engine Gaps & Limitations

This document tracks engine features, APIs, and architectural capabilities identified as missing or limited during the implementation of the `platformer` template game in Phase 1, along with the concrete workarounds adopted in the template.

---

## 1. Scripting API Gaps

### 1.1 Inability to Query Other Entities' Transforms

- **Issue:** `forge.entity.getTransform()` only returns the `Transform2D` of the _currently executing entity_. There is no API like `forge.entity.getTransformOf(entityId: string)` or `forge.scene.findEntity(name: string)`.
- **Impact:** Player, enemies, coins, hazards, and goal flags cannot query each other's live transforms via scripting without custom inter-script messaging or centralized coordination.
- **Workaround:** In `platformer`, the player hero controller script acts as the single primary simulation driver. The level objects (coins, hazards, enemies, checkpoints, flag) are registered with known initial positions in the script configuration or discovered/managed via tilemap layer and coordinated world state. Position updates for enemies and coins are computed or tracked in the level coordinator/player script.

### 1.2 Physics Raycasts and Spatial Queries Return Dummy/Empty Results

- **Issue:** `forge.physics.raycast` returns `None` unconditionally, and `forge.physics.overlapCircle` returns an empty array `[]` in `crates/forge_script/src/bindings.rs`.
- **Impact:** Platformer movement cannot use physics raycasts to detect ground, slopes, or walls.
- **Workaround:** Full kinematic platformer simulation in TypeScript (`scripts/player.ts`). Collision detection with solid ground/platforms is performed directly using world AABB bounds math against the level geometry and tile bounds.

### 1.3 Collision Callbacks (`onCollisionEnter`, `onTriggerEnter`) Not Dispatched

- **Issue:** `rquickjs` module parser checks for `onCollisionEnter`, `onCollisionExit`, and `onTriggerEnter`, but `crates/forge_script/src/systems.rs` does not wire up Rapier physics contact/intersection event listeners from `forge_physics`.
- **Impact:** Script callbacks for physics triggers (sensors) are not automatically invoked.
- **Workaround:** Continuous bounding box (AABB) overlap and radius distance checks in `onUpdate(dt)`.

### 1.4 `forge.animation.play` Is Fire-and-Forget Without SpriteSheet Binding

- **Issue:** Calling `forge.animation.play(clipName)` pushes a command into the queue, but `script_apply_commands_system` does not update the `Sprite.asset_id` or clip on the `Animator`.
- **Workaround:** Sprites with animations can set their UV frames or swap textures directly, or use the procedural sprite sheets with `Animator` component configured in `scene.json`.

---

## 2. UI & Rendering Gaps

### 2.1 Missing In-Game UI / Font Rendering System

- **Issue:** The Forge engine core currently has `wgpu` 2D sprite and tilemap quad renderers (`crates/forge_render`), but does not have a 2D text / font rasterization or UI overlay pipeline (e.g. `wgpu_glyph` or signed distance field text).
- **Impact:** In-game HUD (e.g. coin counter, lives, "Level Complete!" banner) cannot be rendered using a standard UI canvas or text component.
- **Workaround:**
  1. Coin counter and win/checkpoint state are logged in real-time via `forge.log.info()`, visible in the Studio IDE console and test runner stdout.
  2. Visual HUD indicators can use sprite icons (such as collectible coin icons with digit sprites or colored indicator sprites) positioned relative to the camera.

---

## 3. Physics & Character Controller Gaps

### 3.1 `CharacterController2D` Serialization and Script Exposure

- **Issue:** `CharacterController2D` exists in `crates/forge_physics`, but is not serialized in `forge_core::components` and not directly controllable via script API (`forge.entity.setComponent`).
- **Impact:** Built-in slope sliding, autostep, and ground snapping are only available when hardcoded in Rust systems (like `demo_character_controller_system`).
- **Workaround:** Pure kinematic platformer physics written in TypeScript with customizable parameters (gravity, jump velocity, max fall speed, coyote time, jump buffering, variable jump height).

---

## 4. Audio Pipeline Limitations

### 4.1 Bus Volume and Looping Stream Controls

- **Issue:** `forge.audio.play` requires pre-generated or pre-imported `.wav` assets in the project's `assets/` directory. Streaming compressed audio (e.g. `.ogg`, `.mp3`) is not yet supported.
- **Workaround:** Dedicated procedural asset generator (`tools/generate-platformer-assets.ps1` or standalone binary) generates 16-bit PCM WAV files (`jump.wav`, `coin.wav`, `hurt.wav`, `music.wav`) during project initialization.
