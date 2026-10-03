# Forge

Forge is an AI-native game engine and Windows Studio IDE engineered to merge bare-metal Rust performance (`wgpu`, `bevy_ecs`) with autonomous TypeScript-driven agent workflows and a responsive Tauri 2 + React 19 editor.

## Quick Start

### Prerequisites
- Windows 11
- Rust (stable toolchain)
- cargo-nextest (`cargo install cargo-nextest` or download release)
- Node.js 22 LTS
- pnpm (`corepack enable` or `npm install -g pnpm`)

### Setup and Verification

```powershell
# Verify Rust engine workspace
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check

# Install TypeScript dependencies & typecheck
pnpm install
pnpm -r typecheck
pnpm -r lint
pnpm -r test
```

For agent workflows, project guidelines, and full command references, see [AGENTS.md](AGENTS.md).
