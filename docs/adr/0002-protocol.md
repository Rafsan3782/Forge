# ADR 0002: Protocol Schema Definition and Cross-Platform Code Generation

## Context

Forge requires strict, bi-directional IPC contracts between the Rust engine runtime, the Tauri 2 desktop IDE (Studio), and autonomous AI agent runtimes. To avoid drift across language boundaries, a single source of truth is needed for all message definitions (JSON-RPC envelopes, engine ping, scene manipulation, project lifecycle, screenshot capture, and playtest reporting).

Several options were considered for schema definition and code generation:

1. **Rust-first with Serde & ts-rs / specta:**
   - Define types directly in Rust and export TypeScript declarations.
   - _Downside:_ Couples schema authoring to the Rust toolchain and compiler, making it heavier for frontend/agent packages or standalone tools to validate payloads without building Rust.

2. **Protocol Buffers / gRPC / FlatBuffers:**
   - Use binary IDL with official code generators (`protoc`, `prost`).
   - _Downside:_ Adds heavy binary dependencies, compilation complexity across platforms (especially Windows MSVC environments), and makes debugging raw messages over JSON-RPC and WebSocket bridges less transparent during IDE and agent inspection.

3. **JSON Schema as Source of Truth with Typify vs Custom Code Generator:**
   - JSON Schema (`packages/protocol/schemas/*.json`) provides an industry-standard, human-readable, and machine-verifiable contract language with Draft 2020-12 support.
   - _Typify crate:_ `typify` generates robust Rust types from JSON Schema. However, invoking `typify` via build scripts or standalone binaries introduces cross-compilation friction during frontend workflows (e.g., Node/pnpm developers without cargo build tools configured for native linking). Furthermore, `typify` does not produce TypeScript types or runtime Zod validators.
   - _Cross-platform Node Codegen Script:_ A dedicated, deterministic code generator script (`packages/protocol/scripts/codegen.mjs`) runnable with `pnpm run codegen` that parses the canonical JSON Schema files and emits:
     - Strongly typed Rust structs with Serde derives (`Serialize`, `Deserialize`, `#[serde(skip_serializing_if = "...")]`, `#[serde(rename_all = "lowercase")]`) into `crates/forge_protocol/src/generated.rs`.
     - TypeScript interfaces and runtime Zod validation schemas into `packages/protocol/src/generated/index.ts`.

## Decision

We adopt:

1. **JSON Schema as the Canonical Source of Truth:** Stored in `packages/protocol/schemas/`.
2. **Unified, Cross-Platform Node Codegen (`pnpm run codegen`):** A zero-external-binary code generation script that outputs deterministic Rust and TypeScript artifacts marked with `DO NOT EDIT` headers.
3. **Shared JSON Fixtures (`packages/protocol/fixtures/`):** A shared test suite in both Rust (`cargo nextest run -p forge_protocol`) and TypeScript (`pnpm --filter @forge/protocol test`) that verifies roundtrip serialization and deserialization against identical JSON fixtures.
4. **CI Immutability Guard:** CI executes `pnpm run codegen` and asserts `git diff --exit-code`, guaranteeing that generated files are never stale or manually edited.

## Consequences

### Positive

- **Deterministic and Cross-Platform:** The codegen script runs cleanly across Windows, Linux, and macOS without requiring special C/C++ build tools or cargo wrappers during frontend builds.
- **Runtime Validation & Type Safety:** TypeScript receives both compile-time types and runtime Zod validation, ensuring agent actions and IDE inputs conform strictly to the protocol before dispatch.
- **Zero Drift Guarantee:** Both Rust and TypeScript test suites execute against the identical fixture JSON files, verifying wire format compatibility.
- **Fast Iteration:** Updating a schema only requires running `pnpm run codegen` and verifying tests in both languages.

### Negative / Trade-offs

- Schema changes require running the codegen step before commits; enforced by CI check.
- Rust types in `forge_protocol` depend on `serde` and `serde_json`, requiring generated code to align with Serde's attribute conventions.
