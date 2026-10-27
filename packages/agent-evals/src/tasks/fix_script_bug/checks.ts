import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { CheckFn } from "../../types.ts";
import {
  checkForgeCheckClean,
  checkHeadlessRunNoErrors,
  checkFileContains,
} from "../../checks.ts";

export const checks: CheckFn[] = [
  checkForgeCheckClean,
  (ctx) => checkHeadlessRunNoErrors(ctx, 3),
  (ctx) => checkFileContains(ctx, "scripts/coin_spawner.ts", "onUpdate"),
];

export async function goldenSolution(workspaceDir: string): Promise<void> {
  const spawnerScript = path.join(workspaceDir, "scripts/coin_spawner.ts");
  const cleanContent = `// Clean fixed script
let timer: number = 0.0;
const SPAWN_INTERVAL: number = 1.0;
let coinCount: number = 0;

export function onStart(): void {
  forge.log.info("Coin spawner initialized.");
}

export function onUpdate(dt: number): void {
  timer += dt;
  if (timer >= SPAWN_INTERVAL) {
    timer -= SPAWN_INTERVAL;
    coinCount += 1;
    forge.log.info("Spawned new coin: " + coinCount);
  }
}
`;
  await fs.writeFile(spawnerScript, cleanContent, "utf-8");
}
