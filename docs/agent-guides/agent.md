# Agent Guide (`packages/agent`)

## 1. Scope & Ownership

The Agent area contains the autonomous agent runtime, orchestrator, LLM provider integration, safety verification, and tool dispatch located in `packages/agent`:

- `packages/agent/src/contracts`: TypeScript interfaces, in-memory fakes, and test suites for core subsystems (`LlmProvider`, `ToolProvider`, `SafetyPolicy`, `CheckpointManager`, `Orchestrator`, `SkillProvider`, `ProjectBrain`, `DocsProvider`).
- `packages/agent/src/client.ts`: WebSocket client communicating with `forge_bridge`.
- `packages/agent/src/sidecar`: Sidecar runtime communicating with Studio over stdio JSON-RPC.

Tasks working on the agent loop, contracts, or test fakes must restrict edits to `packages/agent/`.

---

## 2. Environment & Commands

Always activate the local toolchain:
```powershell
. .\tools\dev-env.ps1
```

- **Typecheck Agent:**
  ```powershell
  pnpm --filter @forge/agent typecheck
  ```
- **Lint Agent:**
  ```powershell
  pnpm --filter @forge/agent lint
  ```
- **Run Agent Tests:**
  ```powershell
  pnpm --filter @forge/agent test
  ```

---

## 3. Cassette Format Specification

To ensure deterministic, fast, and hermetic testing of model interactions without incurring live API costs or network flakiness, `@forge/agent` uses a JSON cassette format to record and replay Gemini turns.

### Cassette Schema (`.cassette.json`)

```json
{
  "$schema": "https://forge.engine/schemas/cassette.json",
  "version": 1,
  "metadata": {
    "recording_id": "create_platformer_level_turn_1",
    "created_at": "2026-10-26T20:00:00Z",
    "model": "gemini-2.5-pro",
    "temperature": 0.2
  },
  "interactions": [
    {
      "turn_id": "turn_001",
      "request": {
        "messages": [
          { "role": "user", "content": "Create a ground entity with a sprite and box collider." }
        ],
        "available_tools": ["scene.create_entity", "scene.set_component"]
      },
      "response": {
        "thought": "I should create the entity first, then attach the Transform2D and Collider2D components.",
        "tool_calls": [
          {
            "id": "call_001",
            "name": "scene.create_entity",
            "arguments": { "name": "Ground", "parent_id": null }
          }
        ],
        "message_delta": "Creating ground entity...",
        "finish_reason": "tool_calls",
        "usage": {
          "prompt_tokens": 120,
          "completion_tokens": 45,
          "total_tokens": 165
        }
      }
    }
  ]
}
```

### Replay Rules
1. `LlmProvider` fake matches interactions by hash of input messages and tool definitions.
2. If an exact match is found, response deltas and tool calls are yielded deterministically.
3. If no match is found during a test run, an informative error is raised indicating an unrecorded turn.

---

## 4. Conventions

- **Contracts First:** Core agent components must program against interfaces defined in `src/contracts`.
- **Fakes for Testing:** Every contract interface must provide an in-memory fake implementation that operates deterministically without external services.
- **Safety and Checkpoints:** Every mutating operation proposed by an agent must be evaluated by `SafetyPolicy` and preceded by a `Checkpoint`.

---

## 5. Do and Do Not

### Do:
- Use in-memory fakes in contract tests to verify state transitions and event emissions.
- Keep agent events strictly compliant with `@forge/protocol` schemas.
- Provide clear cancellation and pause semantics for the agent loop.

### Do Not:
- Do not make live HTTP calls to external LLM providers in automated tests; use cassettes or fakes.
- Do not mutate the user project without permission if safety policies require confirmation.
- Do not store unencrypted secrets or API keys in repository files.
