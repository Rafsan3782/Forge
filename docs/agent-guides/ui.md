# UI Guide (`packages/ui` & `packages/tokens`)

## 1. Scope & Ownership

The UI area contains the shared design tokens and reusable React component library used across Forge Studio and web applications:

- `packages/tokens`: Design tokens, CSS variables, typography, color palettes, spacing, and theme definitions.
- `packages/ui`: Accessible, unstyled and styled React components (buttons, dialogs, trees, inputs, splitters, toolbars, badges).

Tasks working on design system components or theming own `packages/ui/` and `packages/tokens/`.

---

## 2. Environment & Commands

Always activate the local toolchain:
```powershell
. .\tools\dev-env.ps1
```

- **Typecheck UI & Tokens:**
  ```powershell
  pnpm --filter @forge/ui typecheck
  pnpm --filter @forge/tokens typecheck
  ```
- **Lint UI & Tokens:**
  ```powershell
  pnpm --filter @forge/ui lint
  pnpm --filter @forge/tokens lint
  ```
- **Run Tests:**
  ```powershell
  pnpm --filter @forge/ui test
  ```

---

## 3. Conventions

- **Token Consistency:** All UI components must consume colors, spacing, and font sizes from `@forge/tokens` rather than hardcoding hex codes or pixel offsets.
- **Accessibility:** Ensure interactive components support keyboard navigation, ARIA attributes, and high-contrast modes.
- **Headless & Reusable:** Keep components focused on UI presentation and interaction logic; avoid coupling them directly to engine IPC or backend state.

---

## 4. Do and Do Not

### Do:
- Export clean, strongly typed TypeScript props for all components.
- Provide smooth micro-interactions and dark-mode friendly styling.
- Keep dependencies minimal to ensure fast bundle times in Studio.

### Do Not:
- Do not import `@forge/agent` or engine backend crates into `@forge/ui`.
- Do not use arbitrary CSS class names that conflict with host styles.
- Do not break existing component APIs without coordinating with `apps/studio`.
