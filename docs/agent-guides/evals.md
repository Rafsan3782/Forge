# Agent Evaluation Harness (`packages/agent-evals`)

## 1. Overview & Architecture

The Agent Evaluation Harness evaluates the Forge autonomous agent runtime against structured benchmark tasks across multiple operational modes:

1. **Replay Mode (Deterministic Cassettes):** Uses recorded `.cassette.json` turn transcripts to verify agent logic and checks deterministically on every pull request touching agent packages.
2. **Live Mode (Real Gemini LLM):** Connects to live Gemini models to test agent prompt tuning, real-time reasoning, and autonomous scene generation against live budgets and goals.
3. **Recording Mode:** Runs live tasks and records Gemini's interactions into portable cassettes for hermetic regression tracking.

---

## 2. Benchmark Tasks

The standard suite includes 7 core tasks:
1. `pong`: Make Pong (creates paddles and ball in scene).
2. `double_jump`: Add double jump capability to the platformer player script.
3. `fix_script_bug`: Fix a seeded error in `coin_spawner.ts`.
4. `patrolling_enemy`: Add a patrolling enemy entity with movement script and colliders.
5. `tilemap_level_goal`: Build a tilemap level layout and goal entity.
6. `coin_counter`: Add coin collection counter and HUD display logic.
7. `retune_jump_floatier`: Retune physics jump parameters for floatier gameplay feel.

---

## 3. How to Add a New Eval Task

1. Create a folder in `packages/agent-evals/src/tasks/<task_id>/`.
2. Add `task.yaml`:
   ```yaml
   id: my_task
   name: My Task Name
   description: Detailed description
   template: platformer
   prompt: "User instruction prompt for the agent"
   limits:
     timeout_seconds: 60
     max_turns: 5
     max_tokens: 20000
     max_cost_usd: 0.1
   tags:
     - gameplay
   ```
3. Add `checks.ts`:
   ```typescript
   import type { CheckFn } from "../../types.ts";
   import { checkForgeCheckClean, checkHeadlessRunNoErrors } from "../../checks.ts";

   export const checks: CheckFn[] = [
     checkForgeCheckClean,
     (ctx) => checkHeadlessRunNoErrors(ctx, 5),
   ];

   export async function goldenSolution(workspaceDir: string): Promise<void> {
     // Apply valid modifications to verify checks work
   }
   ```
4. Register the task in `packages/agent-evals/src/tasks/index.ts`.

---

## 4. How to Record a Cassette

To record a live session into a deterministic cassette:

```powershell
. .\tools\dev-env.ps1
node --experimental-strip-types packages/agent-evals/src/cli.ts --live --record
```

This saves `<task_id>.cassette.json` alongside `task.yaml`.

---

## 5. Reading Reports & Comparing Runs

Each evaluation run produces two reports in `eval-reports/`:
- `eval_report.json`: Machine-readable metrics, check status, event timeline, token usage, and cost.
- `eval_report.html`: Interactive dashboard showing pass/fail status per check, timeline events, and final rendered screenshots.

### Comparing Runs

Compare a new run against a baseline to detect regressions in pass rates, steps, or costs:

```powershell
node --experimental-strip-types packages/agent-evals/src/cli.ts --compareWith ./eval-reports/baseline.json
```
