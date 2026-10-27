# Documentation Guide (`docs/*`)

## 1. Scope & Ownership

The Documentation system encompasses one single source of truth (`docs/src/`), automated generators (`tools/docgen`), three distribution outputs (`docs/dist`), architectural records, and agent guidance:

- `docs/src/`: Markdown source pages across `guide/`, `concepts/`, `reference/` (generated), `cookbook/`, `agents/`, and `changelog.md`.
- `tools/docgen/`: TypeScript documentation generators, schema validators, quality gate checkers, and packaging utilities.
- `docs/dist/`: Built distribution targets:
  - `human-bundle/`: Resolved markdown pages and `nav.json` ready for website deployment.
  - `llms.txt` and `llms-full.txt`: Curated and full indexes following the llms.txt standard.
  - `agent-packs/`: Compact briefs under 3000 tokens for autonomous agent fast-path reasoning.
  - `index.json`: Single-lookup machine-readable index with IDs, summaries, and token counts.
  - `search-index.json`: MiniSearch-compatible search index.
  - `docs-bundle.zip`: Compressed archive of all distribution artifacts.
- `docs/ENGINE_STATE.md`: Engine state audit of crates, components, tools, and CI status.
- `docs/adr/`: Numbered Architectural Decision Records.
- `docs/agent-guides/`: Specialized operational guides for each sub-domain.

---

## 2. Environment & Commands

Always activate the local toolchain:
```powershell
. .\tools\dev-env.ps1
```

- **Build documentation:**
  ```powershell
  pnpm docs:build
  ```
- **Validate quality gates (schemas, broken links, code blocks, agent pack sizes):**
  ```powershell
  pnpm docs:validate
  ```
- **Run docgen tests:**
  ```powershell
  pnpm docs:test
  ```

---

## 3. How to Add a Documentation Page

Every page in `docs/src/` must be Markdown with YAML frontmatter conforming to `DocsFrontmatterSchema`:

```markdown
---
id: guide-my-topic
title: "My Topic Title"
summary: "One concise sentence describing the page."
tags: ["guide", "topic"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["guide-getting-started"]
---

# My Topic Title

Content goes here...
```

### Frontmatter Schema Rules:
- `id`: Unique kebab-case string across all documentation pages.
- `title`: Short human-readable title.
- `summary`: Exactly one factual sentence ending with a period.
- `tags`: Non-empty array of topic tags.
- `audience`: `"human"`, `"agent"`, or `"both"`.
- `engine_version`: Semver string (e.g. `"0.1.0"`).
- `last_verified`: Date formatted as `YYYY-MM-DD`.
- `related_ids`: Array of document IDs that must exist in `docs/src/`.

---

## 4. How to Add a Recipe (Cookbook)

Add a markdown file under `docs/src/cookbook/<recipe-name>.md`:
1. Use prefix `cookbook-` for the `id`.
2. Include concrete scene JSON snippets.
3. Include runnable or typechecked TypeScript scripts tagged with mode ````check` or ````run`.
4. Document invariants and common mistakes explicitly.

---

## 5. How Generators Work (`tools/docgen`)

Reference documentation under `docs/src/reference/` is **never hand-edited**. Generators extract definitions directly from the codebase:
- `generate-tools.ts`: Extracts all bridge tools, arguments, return schemas, and error codes from `packages/protocol`.
- `generate-components.ts`: Extracts component fields, types, defaults, ranges, and editor hints from ECS and protocol schemas.
- `generate-events-errors.ts`: Generates notifications and the central error catalog.
- `generate-script-api.ts`: Documents the `forge.*` global script API from `forge.d.ts` and `script.api.json`.
- `generate-cli.ts`: Generates command syntax, options, and examples from `crates/forge_cli`.
- `generate-agent-packs.ts`: Emits self-contained topic briefs into `docs/dist/agent-packs/<topic>.md`, strictly capped at 3000 tokens.

---

## 6. Code Examples & Testing Modes

Fenced code blocks inside documentation pages must specify an execution mode:
- ````check`: TypeScript snippet type-checked against `forge.d.ts` via `forge check`.
- ````run`: Headless simulation run asserting no errors across N frames.
- ````snippet`: Illustration-only snippet without automated execution.

---

## 7. Rules for Writing Agent-Facing Docs

Agent-facing docs require high token efficiency and zero hallucination risk:
1. **Short Sentences:** Avoid compound filler sentences. State facts directly.
2. **Exact Names:** Always use exact tool names (`scene.set_component`) and component names (`Transform2D`).
3. **Runnable Examples:** Every code block must be valid TypeScript conforming to `forge.d.ts`.
4. **State Invariants:** Explicitly document pre-conditions, post-conditions, and frame update boundaries.
5. **Common Mistakes:** Detail exact pitfall cases and their concrete fixes.
6. **No Filler:** Eliminate introductory pleasantries, rhetorical questions, and marketing hype.
