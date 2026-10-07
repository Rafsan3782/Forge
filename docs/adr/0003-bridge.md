# ADR 0003: WebSocket JSON-RPC & MCP Bridge Server and Threat Model

## Context

Forge is designed for real-time collaboration between human creators and autonomous agent workflows. The game engine runtime (`forge_runtime`) must expose runtime inspection, entity manipulation, screenshot capture, and lifecycle control to:
1. The Studio IDE host (Tauri 2 + TypeScript frontend).
2. Developer automation tooling (`forge_cli`).
3. Autonomous AI agents using Model Context Protocol (MCP) or direct JSON-RPC 2.0.

Exposing an inter-process communication (IPC) interface over standard networking sockets introduces severe security risks if misconfigured or exposed to external entities. Specifically, unauthorized clients could hijack the game engine, manipulate ECS scene state, capture sensitive viewport frame buffers, or terminate running sessions.

## Decision

We implement `forge_bridge`, an embedded WebSocket server built with `tokio` and `tokio-tungstenite`, integrated directly into `forge_runtime`, adhering to a strict local-only, token-authenticated threat model.

### 1. Dual Protocol Support (JSON-RPC 2.0 & MCP)

The bridge serves standard JSON-RPC 2.0 envelopes and natively implements the Model Context Protocol (MCP) specification:
- **`tools/list`**: Returns registered tools (`engine.ping`, `scene.get_tree`, `scene.create_entity`, `scene.set_component`, `scene.delete_entity`, `game.screenshot`, `project.stop`) with schemas.
- **`tools/call`**: Executes tool invocations with validated arguments and returns formatted MCP text content blocks.
- **Direct RPC Methods**: Standard direct method dispatching (e.g. `scene.get_tree`, `engine.ping`) for lightweight CLI and editor consumption.

### 2. Lock-Free Frame-Boundary Thread Safety

To guarantee deterministic engine simulation and prevent race conditions or deadlocks:
- Bridge network tasks queue incoming requests via an unbounded MPSC channel (`BridgeCommand`).
- The engine runtime drains pending commands strictly between fixed simulation updates and rendering passes.
- Zero locks or mutexes are held across simulation frames. Responses are delivered asynchronously back to client tasks via oneshot channels.

### 3. Threat Model and Security Controls

The bridge architecture adheres to four fundamental security constraints:

#### A. Strictly Localhost Binding (`127.0.0.1:0`)
- The server binds exclusively to the IPv4 loopback address `127.0.0.1` on an operating-system-assigned random free port (`0`).
- It never binds to `0.0.0.0`, `::`, or any external/public network interface.
- External hosts on the local area network or internet cannot establish TCP connections to the bridge.

#### B. Ephemeral High-Entropy Authentication Token
- A random 32-character hexadecimal token is generated at startup using system timestamp entropy and random hasher state.
- Every client connection must authenticate during the initial HTTP/WebSocket handshake by supplying the token via URL query parameter (`?token=<token>`), URL path (`/<token>`), or `Authorization: Bearer <token>` header.
- Handshakes lacking the token or presenting an invalid token are terminated immediately with `HTTP 401 Unauthorized` before WebSocket protocol upgrade completes.

#### C. Cross-Origin and Browser Hijacking Protection
- Malicious web pages visited in a user's browser may attempt Cross-Site WebSocket Hijacking (CSWSH) against standard loopback ports.
- Because the bridge port is dynamically allocated and requires the high-entropy token generated exclusively inside the runtime process, unauthorized browser scripts cannot guess the port or supply the secret token.

#### D. Local IPC Handshake Protocol
- On successful bind, the runtime writes a single line to standard output:
  `FORGE_BRIDGE_READY {"port":<PORT>, "token":"<TOKEN>"}`
- Host processes (such as Tauri 2, test runners, or `forge_cli`) read this handshake line to discover the dynamic port and token securely without world-readable shared files.

#### E. No Direct Remote Exposure
- Remote debugging or distributed agent fleets across different physical machines must route through authenticated SSH port forwarding or a secured reverse proxy. Direct exposure of the bridge port to public or shared networks is unsupported.

## Consequences

### Positive

- **Autonomous Agent Ready:** Standard MCP clients (Gemini, Claude, Cursor) and Forge CLI can connect to running game sessions out-of-the-box.
- **Robust Isolation:** The engine process is immune to unauthenticated local or network intrusions.
- **Dynamic Port Allocation:** Eliminates port collision issues when running multiple game instances or running tests concurrently.
- **Frame-Safe Determinism:** Queueing commands at frame boundaries eliminates concurrency bugs in the ECS world and GPU pipeline.

### Negative / Trade-offs

- Standalone CLI invocations (`forge call`) require either `--port` and `--token` arguments or discovery via parent process tracking / cached connection info.
- Software fallback adapters are required for headless CI environments that lack physical GPU display drivers.
