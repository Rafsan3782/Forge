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
  (ctx) =>
    checkFileContains(
      ctx,
      "scripts/player.ts",
      /coinCount|coinsCollected|coinTotal|numCoins/i
    ),
];

export async function goldenSolution(workspaceDir: string): Promise<void> {
  const playerScript = path.join(workspaceDir, "scripts/player.ts");
  let content = "";
  try {
    content = await fs.readFile(playerScript, "utf-8");
  } catch {
    // initialize
  }

  if (content.includes("let coinCount: number = 0;")) {
    content = content.replace("let coinCount: number = 0;", "export let coinCount: number = 0;");
  } else if (!content.includes("coinCount")) {
    content = `\n// Coin tracking\nexport let coinCount: number = 0;\n` + content;
  }
  await fs.writeFile(playerScript, content, "utf-8");
}
