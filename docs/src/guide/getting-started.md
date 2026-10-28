---
id: guide-getting-started
title: "Getting Started with Forge"
summary: "Quick start guide for setting up the Forge engine environment, creating a project, and launching the runtime."
tags: ["guide", "getting-started", "setup"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["ref-cli-index", "guide-scripting-basics", "concept-architecture"]
---

# Getting Started with Forge

Forge is an AI-native game engine and Windows Studio IDE engineered for collaborative development between human creators and autonomous agent workflows.

---

## 1. Development Environment

Before running commands, activate the project-local toolchain:

```powershell
. .\tools\dev-env.ps1
```

This ensures Cargo, Node, pnpm, and the `forge` CLI are available in your shell session.

---

## 2. Creating a New Project

Use the `forge new` command to scaffold a project from a template:

```bash
forge new my-game --template platformer
```

This initializes the directory with:
- `scene.json`: Declarative entity hierarchy and components.
- `input_map.json`: Action-to-key bindings.
- `scripts/`: TypeScript gameplay scripts with `forge.d.ts`.
- `assets/`: Procedural textures and audio sound effects.

---

## 3. Running the Project

Launch the interactive game runtime:

```bash
forge run my-game/scene.json
```

To run offscreen headless execution for autonomous agent testing:

```bash
forge run --headless --port 9002
```

---

## 4. Validating Scripts

Verify all TypeScript scripts before running:

```bash
forge check my-game
```

All scripts are checked against the strict compiler settings in `tsconfig.json`.
