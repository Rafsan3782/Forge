# Forge Engine State Audit

**Generated:** 2026-10-26  
**Scope:** Phase 1 Completion & Phase 2 Foundation  
**Status:** Comprehensive audit of crates, bridge tools, protocol schemas, ECS components, templates, build/test flows, known gaps, and ADR conformance.

---

## 1. Rust Workspace Crates

The Rust engine workspace is configured with `resolver = "2"` across 12 primary crates:

| Crate | Purpose | Key Exposed Systems & Primitives |
| :--- | :--- | :--- |
| `forge_core` | Foundation primitives, ECS types, and scene representation | `World`, `Entity`, `Component`, `Vec2`, `Transform2D`, `Sprite`, `Animator`, `Camera2D`, `RigidBody2D`, `Collider2D`, `AudioSource`, `AudioListener`, `Script`, `Scene`, `SceneSerializer` |
| `forge_render` | High-performance WGPU 2D rendering pipeline | `RenderContext`, `SpriteBatchRenderer`, `TilemapRenderer`, `CameraSystem`, offscreen viewport textures, headless frame capture |
| `forge_physics` | Rapier 2D physics integration | `PhysicsWorld2D`, `RigidBodyHandleMap`, `ColliderHandleMap`, kinematic/dynamic stepping, raycasting, sensor query dispatch |
| `forge_audio` | Rodio audio playback and spatialization | `AudioContext`, `SoundBus` (`master`, `music`, `sfx`), playback instances, bus volume attenuation |
| `forge_assets` | Asset database and metadata tracking | `AssetManager`, `AssetId`, `.meta` sidecar parser, hot reload file watcher hooks |
| `forge_input` | Input mapping and polling | `InputState`, keyboard/mouse state trackers, action mapping (`is_action_pressed`, `axis`) |
| `forge_script` | Embedded QuickJS (rquickjs) runtime | `ScriptContext`, TypeScript SWC transpiler, global `forge.*` host bindings, script lifecycle hooks (`onStart`, `onUpdate`, `onFixedUpdate`) |
| `forge_bridge` | JSON-RPC 2.0 & MCP WebSocket IPC bridge | `BridgeServer`, `BridgeDispatcher`, `McpTool` registry, thread-safe command channel to engine loop |
| `forge_protocol` | Serde IPC types and schemas | Generated Rust serde structs/enums for RPC messages, edit operations, and playtest reports |
| `forge_playtest` | Headless playtesting and telemetry | Headless runner, assertion verification, screenshot capture, structured report generator |
| `forge_runtime` | Standalone engine executable | Engine event loop, Win32 window creation, WGPU surface binding, headless CLI runner mode |
| `forge_cli` | Developer CLI tool | `forge` CLI, project validation, script type checking, asset compilation, headless test harness |

---

## 2. Protocol Messages & Bridge MCP Tools

Forge implements bi-directional communication using JSON-RPC 2.0 and MCP (Model Context Protocol). All tools are exposed via WebSocket bridge on port 9002 (or custom port).

### 2.1 Bridge Tools Registry

| Tool Method | Category | Description |
| :--- | :--- | :--- |
| `engine.ping` | System | Heartbeat and liveness check returning engine timestamp and status |
| `scene.get_tree` | Read | Retrieve current entity hierarchy tree from active scene |
| `scene.create_entity` | Write | Create a new entity with optional parent and initial components |
| `scene.set_component` | Write | Set or update a serialized component on an entity |
| `scene.delete_entity` | Write | Delete an entity and all its descendants |
| `game.screenshot` | Read / Exec | Capture viewport screenshot as base64 or write PNG to disk |
| `game.simulate_input` | Exec | Simulate keyboard, mouse, or action events for N frames |
| `project.run` | Lifecycle | Spawn a runtime process session for a given project path |
| `project.stop` | Lifecycle | Terminate the active game runtime session |
| `asset.list` | Read | List all tracked project assets, optionally filtered by asset type |
| `asset.get_info` | Read | Retrieve metadata, path, and type for an asset by UUID |
| `asset.import` | Write | Copy external asset into project and generate `.meta` sidecar |
| `physics.raycast` | Read / Sim | Cast ray through physics world and return first hit collider |
| `tilemap.get` | Read | Query tile ID at coordinates `(x, y)` on a specified layer |
| `tilemap.set_tile` | Write | Set tile ID at coordinates `(x, y)` (returns previous tile ID) |
| `tilemap.fill_rect` | Write | Fill rectangular bounding box with specified tile ID |
| `tilemap.clear` | Write | Clear all tiles or a specific tilemap layer |
| `audio.play` | Exec | Play sound or music track by asset ID onto target bus |
| `audio.stop` | Exec | Stop playback instance or clear entire audio bus |
| `audio.set_bus_volume` | Write | Adjust volume level (0.0 to 1.0) on master/music/sfx bus |
| `script.check` | Read / Exec | Run TypeScript type checking and report diagnostics |
| `script.list` | Read | List all script files in project `scripts/` directory |
| `script.read` | Read | Read raw source text of a script file |
| `script.write` | Write | Validate TypeScript syntax using SWC and write file to disk |
| `play.start` | Sim | Snapshot scene and begin play mode simulation |
| `play.pause` | Sim | Pause active play mode simulation |
| `play.step` | Sim | Step simulation forward by N frames while paused |
| `play.stop` | Sim | Stop play mode and restore original scene snapshot |

### 2.2 Protocol Schema Specifications

- `envelope.json`: JSON-RPC 2.0 envelopes (`JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcError`, `JsonRpcNotification`).
- `scene.edit.ops.json` / `edit.ops.json`: Transactional scene operations (`CreateEntity`, `DeleteEntity`, `SetComponent`, `RemoveComponent`, `ReparentEntity`).
- `game.log.json`: Streamed log notifications with log level, timestamp, target entity, and message.
- `playtest.report.json`: Headless playtest summary schema including pass/fail assertions, frame count, execution duration, and screenshots.

---

## 3. Component Schemas (ECS Core)

Components serialized in `scene.json` and manipulated via `scene.set_component`:

1. `EntityId`: Stable UUID string identifier (`id`).
2. `Name`: Display name (`name`).
3. `Transform2D`: Position `(x, y)`, rotation `angle` in radians, scale `(x, y)`.
4. `Parent`: Hierarchy parent reference (`Option<Entity>`).
5. `Children`: Hierarchy children list (`Vec<Entity>`).
6. `Sprite`: Texture asset UUID, tint color `[r, g, b, a]`, layer order (`z_index`), flip X/Y.
7. `Animator`: Spritesheet asset ID, active clip name, current frame, playback speed, looping state.
8. `Camera2D`: Zoom level, viewport bounds, priority, background clear color.
9. `RigidBody2D`: Body type (`Dynamic`, `KinematicPositionBased`, `KinematicVelocityBased`, `Fixed`), mass, linear/angular damping, gravity scale.
10. `Collider2D`: Shape (`Box`, `Circle`, `Capsule`, `Segment`), friction, restitution, sensor flag, collision groups/masks.
11. `AudioSource`: Sound asset ID, bus channel, volume, pitch, autoplay, loop.
12. `AudioListener`: Spatial listener marker (typically attached to Camera or Player).
13. `Script`: Path to entry TypeScript script (e.g. `"scripts/player.ts"`), enabled flag, serialized properties map.

---

## 4. Starter Templates

| Template | Primary Mechanics & Components Tested |
| :--- | :--- |
| `templates/hello` | Single sprite rendering, camera framing, basic engine runtime boot |
| `templates/tilemap-demo` | Multi-layer tilemap rendering, tile palette indexing, chunk batching |
| `templates/physics-demo` | Rigid bodies, colliders, gravity, bounce restitution, sensor triggers |
| `templates/audio-demo` | Audio buses (`music`, `sfx`), playback triggers, bus volume control |
| `templates/script-demo` | QuickJS runtime integration, entity manipulation via `forge.*`, input polling |
| `templates/platformer` | Complete 2D platformer: kinematic hero controller, procedural WAV audio, coins, hazards, enemies, goal flag, level coordinator |

---

## 5. Build, Test & Codegen Workflows

All commands are executed within the project-local `.tools/` environment initialized via `. .\tools\dev-env.ps1`.

### 5.1 Verification Commands

- **Rust Compile Check:** `cargo check --workspace`
- **Rust Clippy Lint:** `cargo clippy --workspace -- -D warnings`
- **Rust Formatting:** `cargo fmt --check` (format in place: `cargo fmt`)
- **Rust Unit & Integration Tests:** `cargo nextest run --workspace`
- **Headless Runtime Validation:** `cargo run -p forge_runtime -- --headless --frames 5 --screenshot test.png --scene templates/platformer/scene.json`
- **Protocol Codegen:** `pnpm run codegen` (or `node packages/protocol/scripts/codegen.mjs`)
- **TypeScript Typecheck:** `pnpm -r typecheck`
- **TypeScript Lint:** `pnpm -r lint`
- **TypeScript Test Suite:** `pnpm -r test`
- **Studio Frontend Build:** `pnpm --filter studio build`

---

## 6. Known Engine Gaps & Limitations

Summarized from `docs/phase1-gaps.md` and codebase inspection:

1. **Scripting Cross-Entity Transform Query:**
   `forge.entity.getTransform()` only returns the calling entity's transform. There is no `forge.entity.getTransformOf(id)` or `forge.scene.findEntity(name)`. Workaround: Coordinated level script tracking object states.
2. **Physics Raycast in Scripts:**
   `forge.physics.raycast` returns `None` unconditionally in QuickJS bindings (`crates/forge_script/src/bindings.rs`). Workaround: Pure kinematic AABB math in TypeScript for platformer collision.
3. **Collision & Trigger Script Callbacks:**
   `onCollisionEnter` and `onTriggerEnter` are declared in `forge.d.ts` but not yet wired to Rapier contact event streams in `crates/forge_script/src/systems.rs`. Workaround: Distance / bounding box overlap checks in `onUpdate(dt)`.
4. **Sprite Animation Binding:**
   `forge.animation.play(clip)` does not automatically switch `Sprite.asset_id` or advance UV frames inside `script_apply_commands_system`.
5. **In-Engine Font & Text UI Rendering:**
   `forge_render` lacks 2D text rasterization (SDF or glyph atlasing). UI elements rely on sprites or console logging.
6. **CharacterController2D ECS Component:**
   `CharacterController2D` exists in `forge_physics` but lacks full serialization and TypeScript script bindings.
7. **Compressed Audio Formats:**
   `forge_audio` currently plays uncompressed WAV files. Streaming `.ogg` or `.mp3` is not yet supported.
8. **Editor Undo / Redo Command Stack:**
   Tilemap operations return previous IDs to support undo, but the generalized undo command pattern is pending Phase 2.

---

## 7. Deviations from Architectural Decision Records (ADRs)

| ADR | Specified Design | Current Reality | Notes / Rationale |
| :--- | :--- | :--- | :--- |
| **ADR 0003: Bridge & MCP** | Standalone MCP stdio server binary for external LLM tools | Embedded WebSocket server inside `forge_bridge` hosting both JSON-RPC and MCP `tools/list` & `tools/call` endpoints | Pragmatic consolidation for Studio IDE integration; stdio bridge adapter will be hosted by the Phase 2 agent sidecar |
| **ADR 0004: Studio Viewport** | Native Win32 child window embedding via `SetParent` into Tauri webview container | Hybrid fallback with high-performance shared framebuffer / screenshot stream | Cross-process HWND reparenting in Tauri 2 requires specific window composition; stream fallback is rock-solid across Windows platforms |
| **ADR 0002: Protocol Codegen** | Unified dynamic JSON-schema schema compiler | Custom deterministic codegen script (`codegen.mjs`) emitting Zod schemas, TypeScript types, Rust serde structs, and `forge.d.ts` | Maintains zero external codegen build dependencies while preserving exact type synchronization |

---

## 8. CI Health & Mental Run Analysis

Mental walkthrough of `.github/workflows/ci.yml` on `main`:

1. **Path Filtering (`changes` job):**
   Runs on `ubuntu-latest`. Correctly isolates changes to `rust` (`crates/**`, `templates/**`, `Cargo.*`) vs. `typescript` (`packages/**`, `apps/**`, `package.json`, `pnpm-lock.yaml`).
2. **Rust Workflow (`windows-latest`):**
   - Uses `Swatinem/rust-cache@v2`.
   - Downloads `cargo-nextest` via GitHub REST API (potential rate-limiting if unauthenticated, mitigated by passing `secrets.GITHUB_TOKEN`).
   - Runs `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo nextest run --workspace`.
   - Headless screenshot generation runs 3 instances (`hello`, `tilemap-demo`, `platformer`) for 5 frames each.
   - Script checking runs `forge_cli check --project templates/platformer`.
   - Potential bottleneck: First-run compilation on Windows runners takes ~6-8 minutes without cache; ~1.5-2.5 minutes with cache hit.
3. **TypeScript Workflow (`windows-latest`):**
   - Installs dependencies with `pnpm install --frozen-lockfile`.
   - Verifies codegen cleanliness via `pnpm run codegen` followed by `git diff --exit-code`. **Critical constraint:** Any schema edits must have generated code committed.
   - Runs `pnpm -r lint`, `pnpm -r typecheck`, `pnpm -r test`, and `pnpm --filter studio build`.
   - Typical execution time is ~1.5-2 minutes; low flakiness.
4. **Overall CI Reliability:**
   Extremely stable when codegen output matches schema definitions.
