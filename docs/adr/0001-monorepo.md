# ADR 0001: Unified Monorepo Architecture

## Context

Forge consists of multiple interdependent domains:
1. **Core Game Engine:** High-performance systems implemented in Rust (`wgpu`, `bevy_ecs`, audio, physics, asset ingestion).
2. **Studio IDE:** Windows desktop application built with Tauri 2, React 19, and Vite.
3. **Agent Ecosystem & Plugins:** TypeScript packages providing agent runtime, SDKs, UI components, and design tokens.
4. **Inter-Process Protocol:** Strict contracts bridging Rust engine execution and TypeScript editor/agent tooling.

Separating these layers into independent repositories causes significant overhead:
- Schema updates in `@forge/protocol` and `forge_bridge` would require coordinated cross-repo commits and version pinning.
- Local iteration on IDE features that depend on cutting-edge engine capabilities would require complex package linking (`pnpm link` / `cargo path` overrides).
- Continuous Integration would be fragmented, allowing breaking changes to slip past boundaries.

## Decision

We adopt a single unified monorepo containing:
- A **Cargo workspace** (resolver = "2") for all Rust engine crates in `crates/*`.
- A **pnpm workspace** for applications in `apps/*` and packages in `packages/*`.
- Shared documentation, skills, plugins, templates, and architectural decision records.

## Consequences

### Positive
- **Atomic Cross-Stack Commits:** Protocol changes and their corresponding engine and frontend implementations land together in single, verifiable pull requests.
- **Unified CI Validation:** Every push validates Rust formatting, Clippy, tests, TypeScript typechecks, and linters in lockstep on Windows.
- **Simplified Agent Workflow:** Autonomous agents operate with complete repository context without multi-repo synchronization problems.
- **Clear Separation of Concerns:** Workspace boundaries keep Rust crates and TypeScript modules cleanly isolated while sharing standard configuration.

### Negative / Trade-offs
- Monorepos require unified tooling rules and shared dependency management across both Rust and Node ecosystems.
- Workspace-level CI jobs run both Rust and Node toolchains.
