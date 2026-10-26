# Git Workflow Guide

## 1. Scope & Core Rules

Every task and contribution in Forge must follow this strict Git workflow to keep `main` stable and CI-driven:

1. **Never commit directly to `main`:**
   Always start from an up-to-date `main` branch:
   ```powershell
   git switch main
   git pull
   git switch -c <type>/<short-name>
   ```
2. **Small conventional commits:**
   Make atomic, conventional commits on your branch (`feat: ...`, `fix: ...`, `chore: ...`), then push:
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

## 2. Conventions

- **Branch Naming:** Format branches as `<type>/<short-description>`, e.g., `feat/agent-loop`, `fix/tilemap-bounds`, `chore/phase2-foundation`.
- **Commit Messages:** Follow Conventional Commits:
  - `feat: add AgentEvent streaming schema`
  - `fix: correct bounding box calculation in physics raycast`
  - `chore: update dependencies and fakes`
- **Atomic History:** Keep pull requests focused on a single responsibility.

---

## 3. Do and Do Not

### Do:
- Verify working tree is clean before creating a new branch.
- Rebase on latest `main` if conflicts arise.
- Rely on CI for heavy multi-platform validation.

### Do Not:
- Do not force push to `main` under any circumstances.
- Do not squash merge PRs while CI checks are pending or failing.
- Do not leave stale feature branches behind after merging.
