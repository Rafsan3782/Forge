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
      entityName: "LeftPaddle",
    }),
  (ctx) =>
    checkEntityOrComponentExists(ctx, {
      entityName: "RightPaddle",
    }),
  (ctx) =>
    checkEntityOrComponentExists(ctx, {
      entityName: "Ball",
    }),
];

export async function goldenSolution(workspaceDir: string): Promise<void> {
  const scenePath = path.join(workspaceDir, "scene.json");
  let scene: any = { name: "Pong", entities: [] };
  try {
    const raw = await fs.readFile(scenePath, "utf-8");
    scene = JSON.parse(raw);
  } catch {
    // initialize
  }

  scene.entities = scene.entities || [];
  scene.entities.push(
    {
      id: "left_paddle",
      name: "LeftPaddle",
      components: {
        Transform2D: { position: [-350, 0], scale: [1, 1], rotation: 0 },
        Collider2D: { shape: { Box: { half_extents: { x: 10, y: 50 } } } },
      },
    },
    {
      id: "right_paddle",
      name: "RightPaddle",
      components: {
        Transform2D: { position: [350, 0], scale: [1, 1], rotation: 0 },
        Collider2D: { shape: { Box: { half_extents: { x: 10, y: 50 } } } },
      },
    },
    {
      id: "ball",
      name: "Ball",
      components: {
        Transform2D: { position: [0, 0], scale: [1, 1], rotation: 0 },
        Collider2D: { shape: { Circle: { radius: 8 } } },
      },
    }
  );

  await fs.writeFile(scenePath, JSON.stringify(scene, null, 2), "utf-8");
}
