import * as path from "node:path";
import { fileURLToPath } from "node:url";
import type { EvalTaskConfig, CheckFn } from "../types.ts";

// Task 1: Pong
import * as pongChecks from "./pong/checks.ts";
// Task 2: Double Jump
import * as doubleJumpChecks from "./double_jump/checks.ts";
// Task 3: Fix Script Bug
import * as fixBugChecks from "./fix_script_bug/checks.ts";
// Task 4: Patrolling Enemy
import * as patrollerChecks from "./patrolling_enemy/checks.ts";
// Task 5: Tilemap Level Goal
import * as tilemapGoalChecks from "./tilemap_level_goal/checks.ts";
// Task 6: Coin Counter
import * as coinCounterChecks from "./coin_counter/checks.ts";
// Task 7: Retune Jump Floatier
import * as retuneJumpChecks from "./retune_jump_floatier/checks.ts";

export interface LoadedEvalTask {
  config: EvalTaskConfig;
  checks: CheckFn[];
  taskDir: string;
  goldenSolution?: (workspaceDir: string) => Promise<void>;
}

export function getAllTasks(baseDir?: string): LoadedEvalTask[] {
  const currentDir =
    baseDir ||
    (typeof __dirname !== "undefined"
      ? __dirname
      : path.dirname(fileURLToPath(import.meta.url)));

  return [
    {
      config: {
        id: "pong",
        name: "Make Pong Game",
        description: "Create a Pong game with LeftPaddle, RightPaddle, and Ball entities",
        template: "hello",
        prompt: "Create a 2D Pong game in the scene with LeftPaddle, RightPaddle, and Ball entities with colliders.",
        limits: { timeout_seconds: 60, max_turns: 5, max_tokens: 20000, max_cost_usd: 0.1 },
        tags: ["game-creation", "pong", "2d"],
      },
      checks: pongChecks.checks,
      taskDir: path.join(currentDir, "pong"),
      goldenSolution: pongChecks.goldenSolution,
    },
    {
      config: {
        id: "double_jump",
        name: "Add Double Jump to Platformer",
        description: "Add double jump capability to the platformer player script",
        template: "platformer",
        prompt: "Modify the platformer player controller script to allow the player to double jump when airborne.",
        limits: { timeout_seconds: 60, max_turns: 5, max_tokens: 20000, max_cost_usd: 0.1 },
        tags: ["platformer", "player", "mechanics"],
      },
      checks: doubleJumpChecks.checks,
      taskDir: path.join(currentDir, "double_jump"),
      goldenSolution: doubleJumpChecks.goldenSolution,
    },
    {
      config: {
        id: "fix_script_bug",
        name: "Fix Seeded Bug in Script",
        description: "Fix a syntax/type error in scripts/coin_spawner.ts",
        template: "script-demo",
        prompt: "Fix the syntax or type error in scripts/coin_spawner.ts so that type checking and execution succeed.",
        limits: { timeout_seconds: 60, max_turns: 5, max_tokens: 20000, max_cost_usd: 0.1 },
        tags: ["debugging", "typescript"],
      },
      checks: fixBugChecks.checks,
      taskDir: path.join(currentDir, "fix_script_bug"),
      goldenSolution: fixBugChecks.goldenSolution,
    },
    {
      config: {
        id: "patrolling_enemy",
        name: "Add a Patrolling Enemy",
        description: "Add a patrolling enemy entity with movement and collider",
        template: "hello",
        prompt: "Add a patrolling enemy entity named Patroller with a Transform2D, Sprite, and Collider2D to the scene.",
        limits: { timeout_seconds: 60, max_turns: 5, max_tokens: 20000, max_cost_usd: 0.1 },
        tags: ["enemy", "scene", "ai"],
      },
      checks: patrollerChecks.checks,
      taskDir: path.join(currentDir, "patrolling_enemy"),
      goldenSolution: patrollerChecks.goldenSolution,
    },
    {
      config: {
        id: "tilemap_level_goal",
        name: "Build Tilemap Level with Goal",
        description: "Create a tilemap level layout and place a Goal entity",
        template: "tilemap-demo",
        prompt: "Build out the main layer of the tilemap and add a Goal entity with a Collider2D trigger at the end.",
        limits: { timeout_seconds: 60, max_turns: 5, max_tokens: 20000, max_cost_usd: 0.1 },
        tags: ["tilemap", "level-design", "goal"],
      },
      checks: tilemapGoalChecks.checks,
      taskDir: path.join(currentDir, "tilemap_level_goal"),
      goldenSolution: tilemapGoalChecks.goldenSolution,
    },
    {
      config: {
        id: "coin_counter",
        name: "Add Coin Counter UI",
        description: "Add a coin counter variable and collection tracking to player script",
        template: "platformer",
        prompt: "Add a coin counter HUD display and collection tracking in scripts/player.ts.",
        limits: { timeout_seconds: 60, max_turns: 5, max_tokens: 20000, max_cost_usd: 0.1 },
        tags: ["ui", "collectibles", "platformer"],
      },
      checks: coinCounterChecks.checks,
      taskDir: path.join(currentDir, "coin_counter"),
      goldenSolution: coinCounterChecks.goldenSolution,
    },
    {
      config: {
        id: "retune_jump_floatier",
        name: "Retune Jump to Be Floatier",
        description: "Modify GRAVITY and JUMP_VELOCITY parameters in scripts/player.ts",
        template: "platformer",
        prompt: "Retune the jump parameters in scripts/player.ts to be floatier by reducing GRAVITY and adjusting JUMP_VELOCITY.",
        limits: { timeout_seconds: 60, max_turns: 5, max_tokens: 20000, max_cost_usd: 0.1 },
        tags: ["physics", "tuning", "feel"],
      },
      checks: retuneJumpChecks.checks,
      taskDir: path.join(currentDir, "retune_jump_floatier"),
      goldenSolution: retuneJumpChecks.goldenSolution,
    },
  ];
}
