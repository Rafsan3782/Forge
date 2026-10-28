import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { CheckFn } from "../../types.ts";
import {
  checkForgeCheckClean,
  checkHeadlessRunNoErrors,
  checkTilemapTileEquals,
  checkEntityOrComponentExists,
  checkScreenshotVisionRubric,
} from "../../checks.ts";

export const checks: CheckFn[] = [
  checkForgeCheckClean,
  (ctx) => checkHeadlessRunNoErrors(ctx, 3),
  (ctx) => checkTilemapTileEquals(ctx, "main", 0, 0, 1),
  (ctx) =>
    checkEntityOrComponentExists(ctx, {
      entityName: "Goal",
      componentType: "Collider2D",
    }),
  (ctx) =>
    checkScreenshotVisionRubric(
      ctx,
      "The level layout should have platforms leading towards a visible goal area."
    ),
];

export async function goldenSolution(workspaceDir: string): Promise<void> {
  const scenePath = path.join(workspaceDir, "scene.json");
  let scene: { entities?: unknown[] } = { entities: [] };
  try {
    const raw = await fs.readFile(scenePath, "utf-8");
    scene = JSON.parse(raw);
  } catch {
    // initialize
  }

  scene.entities = scene.entities || [];
  scene.entities.push({
    id: "goal_flag",
    name: "Goal",
    components: {
      Transform2D: { position: [400, 64], scale: [1, 1], rotation: 0 },
      Sprite: { asset_id: "hero.png", size: [32, 32], color_tint: [1, 1, 1, 1], z_order: 0 },
      Collider2D: {
        shape: { Box: { half_extents: { x: 16, y: 32 } } },
        is_sensor: true,
      },
    },
  });

  await fs.writeFile(scenePath, JSON.stringify(scene, null, 2), "utf-8");
}
