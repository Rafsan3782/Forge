import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { CheckFn } from "../../types.ts";
import {
  checkForgeCheckClean,
  checkHeadlessRunNoErrors,
  checkEntityOrComponentExists,
} from "../../checks.ts";

export const checks: CheckFn[] = [
  checkForgeCheckClean,
  (ctx) => checkHeadlessRunNoErrors(ctx, 3),
  (ctx) =>
    checkEntityOrComponentExists(ctx, {
      entityName: "Patroller",
      componentType: "Collider2D",
    }),
];

export async function goldenSolution(workspaceDir: string): Promise<void> {
  const scenePath = path.join(workspaceDir, "scene.json");
  let scene: any = { entities: [] };
  try {
    const raw = await fs.readFile(scenePath, "utf-8");
    scene = JSON.parse(raw);
  } catch {
    // initialize
  }

  scene.entities = scene.entities || [];
  scene.entities.push({
    id: "patroller_enemy",
    name: "Patroller",
    components: {
      Transform2D: { position: [200, 80], scale: [1, 1], rotation: 0 },
      Sprite: { asset_id: "hero.png", size: [32, 32] },
      Collider2D: { shape: { Box: { half_extents: { x: 16, y: 16 } } } },
    },
  });

  await fs.writeFile(scenePath, JSON.stringify(scene, null, 2), "utf-8");
}
