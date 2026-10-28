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
      /GRAVITY:\s*number\s*=\s*(?:[1-7]\d{2}(?:\.\d+)?|[1-9]\d(?:\.\d+)?);/
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

  // Replace GRAVITY with floatier value (e.g. 520.0 instead of 980.0)
  if (content.includes("export const GRAVITY: number = 980.0;")) {
    content = content.replace(
      "export const GRAVITY: number = 980.0;",
      "export const GRAVITY: number = 520.0;"
    );
  } else {
    content = "export const GRAVITY: number = 520.0;\nexport const JUMP_VELOCITY: number = 320.0;\n" + content;
  }

  await fs.writeFile(playerScript, content, "utf-8");
}
