# Documentation Guide (`docs/*`)

## 1. Scope & Ownership

The Documentation area encompasses architectural documentation, engineering state audits, Architectural Decision Records (ADRs), and agent guidance:

- `docs/ENGINE_STATE.md`: Factual state of engine crates, tools, components, templates, and CI.
- `docs/adr/`: Numbered Architectural Decision Records (`0001-monorepo.md`, `0002-protocol.md`, etc.).
- `docs/agent-guides/`: Specialized operational guides for each sub-domain.
- `docs/phase1-gaps.md`: Engine limitations, pending items, and adopted workarounds.

Tasks focusing on documentation and architectural records own files under `docs/`.

---

## 2. Environment & Commands

Always activate the local toolchain:
```powershell
. .\tools\dev-env.ps1
```

- **Format check markdown files (if applicable):**
  ```powershell
  pnpm run format:check
  ```
- **Verify links:**
  Ensure relative markdown links to schemas, crates, and ADRs resolve properly.

---

## 3. Conventions

- **Factual & Accurate:** Write technical, factual prose. Avoid buzzwords, hype, or marketing language.
- **Keep Summaries Compact:** High-level audits (such as `ENGINE_STATE.md`) must remain structured, skimmable, and strictly under 600 lines.
- **ADR Format:** Follow standard ADR structure: Title, Status, Context, Decision, Consequences, and Implementation Notes.
- **Forward-Looking & Traceable:** Cross-reference ADRs and guides when decisions impact multiple packages.

---

## 4. Do and Do Not

### Do:
- Update `ENGINE_STATE.md` when new crates, bridge tools, or components are added.
- Document known engine gaps and concrete workarounds clearly.
- Create new numbered ADRs when introducing fundamental structural changes.

### Do Not:
- Do not duplicate guide content between `AGENTS.md` and domain guides under `docs/agent-guides/`.
- Do not let documentation drift from the actual checked-in code.
- Do not commit speculative documentation without marking its status as Proposed or Draft.
