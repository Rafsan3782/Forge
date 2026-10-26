# ADR 0005: Autonomous Agent Architecture & Studio Integration

## Status

Accepted

## Context

Forge is an AI-native game engine designed from the ground up for collaborative real-time creation between human developers and autonomous agent workflows. Phase 1 established the high-performance Rust engine core (`forge_core`, `forge_render`, `forge_physics`, `forge_audio`, `forge_script`, `forge_bridge`), the JSON-RPC/MCP bridge, and the Tauri 2 Studio IDE.

Phase 2 introduces the autonomous agent execution loop powered by Google Gemini. The agent must be capable of understanding multi-modal project state (scene hierarchy, scripts, assets, viewport screenshots, console diagnostics), formulating structured multi-step plans, requesting user permissions for risky operations, and executing commands reliably.

Key architectural challenges include:
1. **Host Decoupling & Isolation:** Running long-lived agent execution loops and LLM orchestration inside the UI thread of Tauri Studio risks UI freezes, memory bloat, and crashes.
2. **Deterministic & Reversible Actions:** Autonomous agent actions (such as generating code or modifying scene graphs) must be checkpointed and reversible so users can inspect, accept, or revert changes.
3. **Multi-Model Orchestration & Tool Dispatch:** Agents need structured access to engine tools via standard protocols (MCP / JSON-RPC).
4. **Session Persistence:** Agent reasoning history, tool invocations, checkpoint states, and budget telemetry must persist across Studio restarts.

---

## Decision

We establish the Phase 2 agent architecture on the following foundations:

### 1. TypeScript Sidecar Process Next to Studio
The agent runtime is implemented as an independent TypeScript sidecar process (`@forge/agent`) spawned and managed by the Studio Tauri host.
- The sidecar runs in an isolated Node.js runtime.
- Studio communicates with the agent sidecar via **JSON-RPC 2.0 over standard I/O (stdio)**, receiving a real-time reactive **`AgentEvent` stream** (`run_started`, `message_delta`, `tool_call_started`, `permission_requested`, `checkpoint_created`, `budget_update`, etc.).
- The sidecar exposes an **`AgentCommand` interface** (`start_run`, `send_message`, `stop`, `pause`, `resume`, `resolve_permission`, `revert_checkpoint`, `set_settings`).

### 2. Engine Interaction via WebSocket Bridge (MCP)
The agent sidecar connects to the running Forge engine via the WebSocket bridge server (`forge_bridge`).
- The sidecar communicates with the engine using the **Model Context Protocol (MCP)** (`tools/list`, `tools/call`).
- Engine capabilities (scene manipulation, screenshot capture, physics raycasting, tilemap editing, audio playback, script analysis) are exposed as standard MCP tools.

### 3. Session Persistence via SQLite
Agent session state, user prompts, assistant thoughts, tool call inputs/outputs, and cost/token budget telemetry are recorded in a project-local SQLite database (`.forge/agent_sessions.db`).
- Allows seamless resumption of agent runs across IDE restarts.
- Provides an immutable audit trail of agent actions and planning steps.

### 4. Git Checkpoints Per Step
Every mutating plan step proposed or executed by the agent creates an atomic Git checkpoint:
- Before executing modifying tools (`script.write`, `scene.set_component`, `tilemap.fill_rect`), a temporary Git commit or ref is recorded.
- Users can preview unified diffs before granting permissions.
- If an agent error occurs or the user rejects the changes, the project can be reverted instantly using the `revert_checkpoint` command.

---

## Consequences

### Positive
- **Fault Isolation:** An agent crash, timeout, or runaway loop does not crash the Studio IDE or engine runtime.
- **Protocol Consistency:** Uses standard MCP and JSON-RPC 2.0 schemas defined in `@forge/protocol`.
- **Fast, Hermetic Testing:** Subsystems (`LlmProvider`, `ToolProvider`, `SafetyPolicy`, `CheckpointManager`) are abstracted behind TypeScript contracts with deterministic in-memory fakes and JSON cassette recordings.
- **Safety & Trust:** Granular permission modes (`autonomous`, `ask_risky`, `ask_all`) paired with git checkpoints guarantee user control over their project.

### Negative / Trade-offs
- Requires managing the lifecycle of an auxiliary sidecar process alongside Tauri.
- Stdio serialization introduces minimal overhead, negligible compared to LLM network latency.
