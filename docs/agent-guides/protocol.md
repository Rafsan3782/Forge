# Protocol Guide (`packages/protocol` & `crates/forge_protocol`)

## 1. Scope & Ownership

The Protocol area is the single source of truth for all IPC messages, schemas, and data structures shared between the Rust engine, Studio IDE, and autonomous agents:

- `packages/protocol/schemas`: JSON Schema specifications for requests, responses, notifications, and events.
- `packages/protocol/scripts/codegen.mjs`: Code generator emitting TypeScript Zod schemas, TypeScript types, and Rust Serde structs.
- `packages/protocol/src/generated`: Generated TypeScript schemas and types (DO NOT HAND EDIT).
- `crates/forge_protocol/src/generated.rs`: Generated Rust serde definitions (DO NOT HAND EDIT).

Tasks modifying IPC contracts or schemas own `packages/protocol/schemas/` and `packages/protocol/scripts/codegen.mjs`.

---

## 2. Environment & Commands

Always activate the local toolchain:
```powershell
. .\tools\dev-env.ps1
```

- **Run code generation:**
  ```powershell
  pnpm run codegen
  ```
- **Typecheck protocol package:**
  ```powershell
  pnpm --filter @forge/protocol typecheck
  ```
- **Lint protocol package:**
  ```powershell
  pnpm --filter @forge/protocol lint
  ```
- **Run protocol tests:**
  ```powershell
  pnpm --filter @forge/protocol test
  cargo nextest run -p forge_protocol
  ```

---

## 3. Conventions

- **Schema as Source of Truth:** Never add a message type directly into Rust or TypeScript code without defining its JSON Schema under `packages/protocol/schemas/` first.
- **Strict Backward Compatibility:** Protocol evolution should be additive whenever possible. Deprecate fields before removing them.
- **Envelope Standardization:** All commands and responses must conform to JSON-RPC 2.0 or the MCP protocol envelope.
- **Event Streaming:** Autonomous agent streaming events conform to the `AgentEvent` schema family.

---

## 4. Do and Do Not

### Do:
- Run `pnpm run codegen` immediately after modifying any `.json` schema in `packages/protocol/schemas/`.
- Verify that `git diff` shows clean generated files in both Rust and TypeScript destinations.
- Write schema validation unit tests in `packages/protocol/src/protocol.test.ts`.

### Do Not:
- Do not manually edit files inside `packages/protocol/src/generated/` or `crates/forge_protocol/src/generated.rs`.
- Do not commit changes to schemas without committing the regenerated TypeScript and Rust files.
- Do not add runtime dependencies to `@forge/protocol` other than Zod.
