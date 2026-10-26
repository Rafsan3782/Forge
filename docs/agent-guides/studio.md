# Studio Guide (`apps/studio`)

## 1. Scope & Ownership

The Studio area comprises the desktop IDE application built with Tauri 2, React 19, TypeScript, and Vite located in `apps/studio`:

- `apps/studio/src`: React UI components, editor panels, scene tree, inspector, console, asset browser, and viewport container.
- `apps/studio/src-tauri`: Tauri 2 Rust host application, window management, native Win32 embedding, and process lifecycle.

Tasks working on the IDE frontend or Tauri shell must restrict edits to `apps/studio/`.

---

## 2. Environment & Commands

Always activate the local toolchain:
```powershell
. .\tools\dev-env.ps1
```

- **Install dependencies:**
  ```powershell
  pnpm install
  ```
- **Typecheck Studio:**
  ```powershell
  pnpm --filter studio typecheck
  ```
- **Lint Studio:**
  ```powershell
  pnpm --filter studio lint
  ```
- **Build Studio Frontend:**
  ```powershell
  pnpm --filter studio build
  ```
- **Run Studio in dev mode (Vite):**
  ```powershell
  pnpm --filter studio dev
  ```

---

## 3. Conventions

- **State Management:** Keep editor state modular; use shared hooks and bridge client subscriptions.
- **IPC Communication:** Communicate with the engine runtime exclusively via `BridgeClient` (@forge/agent or @forge/protocol contracts) over WebSocket.
- **Component Styling:** Use the `@forge/ui` shared component library and `@forge/tokens` design tokens. Do not introduce ad-hoc styling systems.
- **Resilience:** Gracefully handle engine disconnections, crashes, or reconnects with clear status indications in the Studio status bar.

---

## 4. Do and Do Not

### Do:
- Validate props and incoming bridge data against `@forge/protocol` Zod schemas.
- Provide clean keyboard shortcuts matching standard IDE expectations.
- Ensure the viewport container properly requests and displays engine frames or embeds the Win32 window.

### Do Not:
- Do not make direct Rust engine FFI calls from the webview; route all engine commands through the protocol bridge.
- Do not store heavy binary assets in frontend state.
- Do not hardcode engine connection ports; load from config or runtime session discovery.
