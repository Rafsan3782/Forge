---
id: agents.rules-and-invariants
title: Engine Rules, Invariants, and Troubleshooting Catalog
summary: Mandatory architectural invariants and a comprehensive 30+ catalog of common mistakes, causes, and fixes.
tags: ["agents", "invariants", "rules", "troubleshooting", "errors"]
audience: agent
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.mental-model", "agents.conventions", "agents.limits"]
---

# Rules, Invariants, and Troubleshooting

## 1. Engine Invariants (Must ALWAYS Be True)

1. **Cartesian Coordinates:** `+Y` is up, `-Y` is down. Gravity points in `-Y`.
2. **Transform Completeness:** Every renderable or physical entity must possess a `Transform2D` component.
3. **Hierarchy Integrity:** An entity cannot be its own parent or ancestor (cycles are illegal in ECS scene graphs).
4. **Stable Entity IDs:** Every entity in `scene.json` must have a unique, non-empty `id` string (e.g. `"player_hero"`).
5. **Single Script Controller Context:** `forge.entity.getTransform()` only accesses the calling entity's own transform. Cross-entity queries must be coordinated centrally.
6. **WAV Audio Only:** All audio assets must be uncompressed 16-bit PCM `.wav` files.
7. **Asset Sidecar Parity:** Every file in `assets/` must have an associated `.meta.json` sidecar.
8. **Camera Necessity:** Every scene must contain at least one entity with an active `Camera2D` component, or the viewport will render a blank clear color.
9. **No Modifying Generated Code:** Never hand edit `forge.d.ts` or generated protocol schemas.
10. **Zero Compile Errors:** Scripts with TypeScript errors reported by `script.check` will fail to execute in QuickJS.

---

## 2. Common Mistakes & Troubleshooting Catalog

| # | Symptom | Root Cause | Exact Fix |
| :--- | :--- | :--- | :--- |
| **1** | Character falls upward into the sky. | Gravity scale or jump velocity configured with negative sign when expecting downwards gravity. | Set downward gravity in `-Y` (e.g. `const GRAVITY = 980.0; velY -= GRAVITY * dt;`). |
| **2** | Screen is completely black or solid clear color. | No entity in `scene.json` has an active `Camera2D` component or camera zoom is `0.0`. | Add a camera entity with `Transform2D: { position: [x, y], rotation: 0, scale: [1, 1] }` and `Camera2D: { zoom: 1.0, is_active: true }`. |
| **3** | Character falls straight through platform/floor. | Physics collision disabled, colliders missing, or relying on unimplemented script raycasts. | Ensure solid platforms are defined as rigid bodies/colliders or use kinematic AABB ground checks against platform rects. |
| **4** | Script crashes on launch: `TypeError: cannot read property 'position' of null`. | Entity lacks a `Transform2D` component. | Attach `Transform2D` component in `scene.json` or call `forge.entity.setComponent("Transform2D", { position: [0, 0], rotation: 0, scale: [1, 1] })`. |
| **5** | Sound does not play when calling `forge.audio.play("jump.wav")`. | Audio asset not imported into `assets/`, missing `.meta.json`, or bus volume is 0. | Import `jump.wav` into `assets/`, check `asset.list`, and verify `forge.audio.setBusVolume("sfx", 1.0)`. |
| **6** | Sprite is invisible despite valid coordinates. | `z_index` on `Sprite` is lower than background tilemap layer, or sprite texture UUID is empty. | Set `z_index: 10` on `Sprite` and ensure `asset_id` links to a valid texture asset. |
| **7** | Sprite is cropped or distorted. | Scale component set to `[0, 0]` or non-uniform negative scale. | Set `scale: [1.0, 1.0]`. Use `flip_x: true` on `Sprite` instead of negative scale for mirroring. |
| **8** | Calling `forge.entity.getTransformOf("enemy")` throws undefined function error. | API does not exist; scripts only access their host entity. | Manage shared positions via central coordinator script or script properties. |
| **9** | Player teleporting wildly on frame stutters. | Delta time `dt` used directly without clamping during initialization lag. | Clamp delta time: `const safeDt = Math.min(dt, 0.05);`. |
| **10** | Jump height varies drastically at different framerates. | Gravity and velocity integrated without multiplying by delta time. | Multiply both velocity and acceleration by `dt`: `posY += velY * dt; velY -= GRAVITY * dt;`. |
| **11** | Key presses not detected by `forge.input.isActionPressed("jump")`. | Action name `"jump"` not registered in `input_map.json`. | Add action mapping in `input_map.json`: `{"actions": {"jump": [{"type": "Key", "code": "Space"}]}}`. |
| **12** | Tilemap displays wrong tiles or blank chunks. | Tile coordinates in `Tilemap.chunks` out of bounds or tileset palette mismatch. | Ensure chunk coordinates match `[chunkX, chunkY]` and tile IDs exist in `tileset.json`. |
| **13** | Physics raycast returns `null` every time. | Engine QuickJS binding returns `None` unconditionally in Phase 1. | Use TypeScript kinematic AABB collision math against platform arrays. |
| **14** | `onCollisionEnter` never gets called. | Rapier contact callbacks not yet wired to QuickJS script hooks. | Perform AABB distance overlap checks in `onUpdate(dt)`. |
| **15** | Enemies walk through walls or patrol off screen. | Patrol limits not clamped or horizontal reversal condition never met. | Check bounds in update: `if (x >= RIGHT_BOUND) dir = -1; else if (x <= LEFT_BOUND) dir = 1;`. |
| **16** | Background music plays multiple overlapping instances. | `forge.audio.play("music.wav")` called inside `onUpdate` instead of `onStart`. | Move audio play call into `onStart()` and set `{ loop: true }`. |
| **17** | Text/HUD component fails to deserialize in scene. | No built-in font/text component exists in Phase 1 engine. | Render HUD numbers and health bars using sprites or log values via `forge.log.info`. |
| **18** | Screen shake causes permanent camera drift. | Shake offset applied permanently to camera base transform. | Store `baseCameraPosition` separately and apply shake offset as a transient modifier. |
| **19** | Coins collected count increments by 60 per second. | Coin collision check doesn't mark coin as `collected = true` on first overlap. | Set `coin.collected = true` immediately and play collection SFX only once. |
| **20** | `forge.spawn()` creates entity but it doesn't render. | Spawned entity has no `Transform2D` or `Sprite` component assigned. | Provide initial components: `forge.spawn("Coin", { Transform2D: { position: [x, y] }, Sprite: { asset_id: "coin.png" } })`. |
| **21** | Destroyed entity still collides or runs logic. | Entity destroyed in script but local array reference still kept in simulation loop. | Remove entity from local array or check if entity is valid before processing. |
| **22** | Moving platform leaves player behind. | Platform moves but does not add platform velocity to grounded player. | When `isGroundedOnPlatform`, add `platformVelX * dt` to player position. |
| **23** | Player gets stuck inside walls when walking into them. | Position updated directly without separating X and Y axis collision resolution. | Move X axis -> resolve X collision -> Move Y axis -> resolve Y collision. |
| **24** | Jump buffering triggers a jump seconds after pressing space. | Jump buffer timer not decremented each frame. | Decrement timer: `jumpBufferTimer = Math.max(0, jumpBufferTimer - dt);`. |
| **25** | Coyote time allows jumping in mid-air infinitely. | Coyote timer reset every frame even when not grounded. | Only set `coyoteTimer = COYOTE_TIME` when `isGrounded` is strictly true; decrement otherwise. |
| **26** | Audio volume adjustment has no effect. | Volume set to values outside `[0.0, 1.0]` or bus name misspelled. | Use exact bus strings: `"master"`, `"music"`, or `"sfx"` and volume between `0.0` and `1.0`. |
| **27** | Game won state triggers immediately on level start. | Goal flag position set at `(0, 0)` which matches initial player spawn. | Set goal flag at far end of the level (e.g. `x = 1440.0, y = 80.0`). |
| **28** | Respawning at checkpoint keeps old velocity and falls into hazard. | Velocity not reset to zero upon respawn. | Reset `velX = 0; velY = 0;` whenever resetting `posX` and `posY` to checkpoint. |
| **29** | Camera jerks violently when player jumps. | Camera smoothing speed set too high (e.g. `> 30.0`) or instant snapping used. | Use exponential damping: `current = lerp(current, target, 1.0 - Math.exp(-smoothSpeed * dt));`. |
| **30** | Sprite renders behind tilemap ground. | Ground tilemap layer renders at `z_index = 0` while sprite is at `z_index = -1`. | Assign sprites a higher `z_index` (e.g. `z_index: 5`) than background layers. |
| **31** | Script changes in editor don't take effect in play mode. | Script file not saved to disk or syntax errors prevent compilation. | Save via `script.write` and verify no diagnostics with `script.check`. |
| **32** | Infinite loop crashes engine during headless run. | `while` loop inside `onUpdate` lacking loop termination condition. | Use frame-based timers instead of blocking `while` loops in script callbacks. |
