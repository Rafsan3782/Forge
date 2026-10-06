# AGENTS.md

## Project Summary

**Forge** is an AI-native game engine and Windows Studio IDE designed for real-time collaboration between human creators and autonomous agent workflows.

- **Engine Core:** High-performance systems built in Rust leveraging `wgpu` for graphics and `bevy_ecs` for entity-component orchestration.
- **Studio IDE:** Desktop application built with Tauri 2, React 19, TypeScript, and Vite.
- **Agents & Tooling:** TypeScript runtime and packages powering autonomous workflows, code generation, and scene manipulation.

---

## Architecture Map

```text
forge/
├── .tools/                       # Git-ignored local toolchains & runtime caches
├── apps/                         # Applications (e.g. Studio Tauri 2 + Vite frontend)
├── crates/                       # Rust engine workspace (resolver = "2")
│   ├── forge_core/               # Engine foundation primitives & math
│   ├── forge_render/             # wgpu rendering pipeline
│   ├── forge_physics/            # Physics simulation integration
│   ├── forge_audio/              # Audio playback & spatialization
│   ├── forge_assets/             # Asset pipeline & streaming
│   ├── forge_input/              # Input mapping & polling
│   ├── forge_script/             # Scripting runtime & VM hooks
│   ├── forge_bridge/             # IPC bridge between Rust engine and IDE host
│   ├── forge_protocol/           # Shared protocol schemas & serde IPC types
│   ├── forge_playtest/           # Headless playtesting & telemetry
│   ├── forge_runtime/            # Standalone game runtime binary
│   └── forge_cli/                # Developer command-line tool binary
├── packages/                     # TypeScript workspace (@forge/*)
│   ├── protocol/                 # Schema definitions & IPC contracts
│   ├── agent/                    # AI agent logic, tools, and execution loop
│   ├── plugin-sdk/               # Public SDK for engine & editor plugins
│   ├── ui/                       # Shared React component library
│   └── tokens/                   # Design tokens & theming
├── skills/                       # Agent skills and instruction sets
├── plugins/                      # Engine/Editor plugins
├── templates/                    # Starter templates for game projects
├── tools/                        # Automation and developer utility scripts
└── docs/adr/                     # Architectural Decision Records
```

---

## Project-Local Tooling (`.tools/`)

All development toolchains and caches for this repository live inside the git-ignored `.tools/` directory at the repository root. No global or system PATH changes are made.

### Session Initialization

Every PowerShell terminal session **must start by activating the local environment**:

```powershell
. .\tools\dev-env.ps1
```

To verify that all tools resolve strictly inside `.tools/`, run:

```powershell
.\tools\dev-env-check.ps1
```

### Directory Layout

- `.tools/cargo/`: Cargo home (`CARGO_HOME`), registry, and binaries (`cargo`, `rustc`, `rustup`, `cargo-nextest`).
- `.tools/rustup/`: Rustup home (`RUSTUP_HOME`) and installed toolchain (`stable-x86_64-pc-windows-msvc`).
- `.tools/node/`: Isolated Node.js runtime copied from host.
- `.tools/npm-global/`: Local `pnpm` CLI and wrapper scripts.
- `.tools/pnpm-store/`: pnpm package store directory.
- `.tools/corepack/`: Isolated Corepack cache (`COREPACK_HOME`).
- `.tools/git/`: Git for Windows runtime copied from host.
- `.tools/gh/`: Standalone GitHub CLI (`gh`) binary.

### System Dependencies

The following host-level components remain external:

- **MSVC CRT & Windows SDK:** Headers and libraries accessed via project-local `.xwin-cache/splat` and linked via `rust-lld`.
- **Microsoft Edge WebView2 Runtime:** System-level runtime (`msedgewebview2.exe`) used by Tauri.
- **GPU Display Drivers:** DirectX 12 / Vulkan kernel display drivers for `wgpu`.

### Manual PATH Cleanup (Action for User)

You can safely remove the following project-specific entry from your Windows User Environment Variables `PATH`:

- `C:\Users\mdrih\.cargo\bin`

_(Note: Pre-existing system entries such as `C:\Program Files\nodejs`, `C:\Program Files\Git\cmd`, and `C:\Users\mdrih\AppData\Roaming\npm` were copied rather than moved and should remain in system PATH if other projects on your machine rely on them.)_

---

## Git Workflow

Every future task and contribution must strictly follow this workflow:

1. **Never commit directly to `main`:**
   Always start from an up-to-date `main` branch:
   ```powershell
   git switch main
   git pull
   git switch -c <type>/<short-name>
   ```
2. **Small conventional commits:**
   Make atomic, conventional commits on your branch, then push:
   ```powershell
   git push -u origin <type>/<short-name>
   ```
3. **Open a Pull Request:**
   - Use GitHub CLI (`gh pr create`) if authenticated.
   - If `gh` is not authenticated, output the exact PR title and body for manual creation.
4. **CI-driven validation (Do not run heavy suites locally):**
   - Do **not** run the full test suite, clippy, or cargo test locally; CI executes them on Windows runners.
   - Run locally only what CI cannot check, and only when the task explicitly requires it (e.g. manual/visual checks).
5. **Monitor CI & Handle Failures:**
   - Wait for CI checks to run.
   - If CI fails, inspect the failed job logs, fix on the same branch, and push again.
   - After 3 consecutive failed attempts, stop and report the blocking issue to the user.
6. **Squash Merge & Branch Cleanup:**
   - Once CI is green:
     - If GitHub CLI is authenticated: squash merge using the agreed commit message (`gh pr merge --squash --delete-branch`) and delete the branch.
     - If not authenticated: provide the PR link, the exact squash commit subject and body, and instruct the user to press "Squash and merge".
   - After merge, return to main and clean up locally:
     ```powershell
     git switch main
     git pull
     git branch -d <type>/<short-name>
     ```
7. **Task Completion Summary:**
   The final message of every task must list:
   - Branch name
   - PR link
   - CI status
   - Merge commit message
   - Any manual verification steps needed

---

## Exact Build, Test, and Lint Commands

All commands assume the session was activated with `. .\tools\dev-env.ps1`:

### Rust Workspace

Run from the repository root:

- **Typecheck & compile check:**
  ```powershell
  cargo check --workspace
  ```
- **Code formatting check:**
  ```powershell
  cargo fmt --check
  ```
- **Format code in-place:**
  ```powershell
  cargo fmt
  ```
- **Linter (warnings as errors):**
  ```powershell
  cargo clippy --workspace -- -D warnings
  ```
- **Run tests with cargo-nextest:**
  ```powershell
  cargo nextest run
  ```
- **Run protocol tests only:**
  ```powershell
  cargo nextest run -p forge_protocol
  ```

### TypeScript Workspace

Run from the repository root:

- **Install dependencies:**
  ```powershell
  pnpm install
  ```
- **Run protocol code generation:**
  ```powershell
  pnpm run codegen
  ```
- **Typecheck all packages:**
  ```powershell
  pnpm -r typecheck
  ```
- **Lint all packages:**
  ```powershell
  pnpm -r lint
  ```
- **Run all package tests:**
  ```powershell
  pnpm -r test
  ```
- **Run protocol tests only:**
  ```powershell
  pnpm --filter @forge/protocol test
  ```
- **Check code formatting:**
  ```powershell
  pnpm run format:check
  ```
- **Format code in-place:**
  ```powershell
  pnpm run format
  ```

---

## Agent Operational Rules

1. **Protocol First, Then Both Sides:**
   - Any cross-boundary communication (Rust Engine $\leftrightarrow$ TypeScript IDE / Agent) MUST begin with updating schemas in `@forge/protocol` and `forge_bridge`.
   - Never implement engine-side or frontend-side features in isolation without syncing the protocol contract first.

2. **Never Edit Generated Files:**
   - Do not manually edit files generated by compilers, code generators, build scripts, or asset processors.
   - Modify the source schema, generator template, or configuration instead.

3. **Small Commits:**
   - Keep atomic commits with clear, descriptive commit messages.
   - Avoid giant multi-purpose PRs or edits.

4. **Definition of Done (DoD):**
   - The workspace builds cleanly (`cargo check --workspace`).
   - All tests pass (`cargo nextest run` and `pnpm -r test`).
   - Zero new compiler, clippy, or ESLint warnings (`cargo clippy --workspace -- -D warnings` and `pnpm -r lint`).
   - All TypeScript types pass strict validation (`pnpm -r typecheck`).
