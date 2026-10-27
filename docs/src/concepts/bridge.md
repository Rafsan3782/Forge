---
id: concepts.bridge
title: JSON-RPC and MCP Bridge Protocol
summary: WebSocket IPC bridge connecting the Rust runtime with the Studio IDE and autonomous agent workflows.
tags: ["concepts", "bridge", "mcp", "json-rpc", "protocol", "ipc"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-26"
related_ids: ["agents.the-work-loop", "concepts.ecs"]
---

# Bridge Protocol & MCP

Forge exposes its internal engine state to the Studio IDE and autonomous agents through an embedded JSON-RPC 2.0 and MCP (Model Context Protocol) WebSocket server running in `forge_bridge`.

## IPC Structure

- **Transport:** WebSocket server bound to `ws://127.0.0.1:9002` (configurable).
- **Authentication:** Bearer token stored in `.forge/bridge.json`.
- **Message Types:**
  - `JsonRpcRequest`: Client invocation of engine tools with `id`, `method`, `params`.
  - `JsonRpcResponse`: Engine tool execution results.
  - `JsonRpcError`: Structured errors with error codes, messages, and remedial hints.
  - `JsonRpcNotification`: Engine event broadcasts (e.g. `game.log`, scene changes).
- **Model Context Protocol (MCP):** Exposes `tools/list` and `tools/call` for direct integration with AI agents.
