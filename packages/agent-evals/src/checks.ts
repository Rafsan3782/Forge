import * as fs from "node:fs/promises";
import * as path from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import type { CheckResult, CheckContext } from "./types.ts";

const execFileAsync = promisify(execFile);

interface SceneEntityLike {
  id?: string;
  name?: string;
  components?: Record<string, Record<string, unknown>>;
}

interface SceneJsonLike {
  entities?: SceneEntityLike[];
}

function getErrorMessage(err: unknown): string {
  if (err instanceof Error) {
    const execErr = err as Error & { stdout?: string; stderr?: string };
    return execErr.stdout || execErr.stderr || execErr.message;
  }
  return String(err);
}

/**
 * 1. Checks that forge check runs cleanly without TypeScript or script errors on the project.
 */
export async function checkForgeCheckClean(ctx: CheckContext): Promise<CheckResult> {
  const checkName = "forge check is clean";
  try {
    const tsconfigPath = path.join(ctx.workspaceDir, "tsconfig.json");
    try {
      await fs.access(tsconfigPath);
    } catch {
      return {
        name: checkName,
        passed: true,
        message: "No tsconfig.json in project, check skipped as clean",
      };
    }

    const { stdout } = await execFileAsync("cargo", [
      "run",
      "-p",
      "forge_cli",
      "--",
      "check",
      "--project",
      ctx.workspaceDir,
    ]);

    return {
      name: checkName,
      passed: true,
      message: stdout || "Typecheck passed cleanly",
    };
  } catch (error: unknown) {
    return {
      name: checkName,
      passed: false,
      message: `forge check failed: ${getErrorMessage(error)}`,
    };
  }
}

/**
 * 2. Checks that the engine runtime headless mode renders N frames without runtime errors or crashes.
 */
export async function checkHeadlessRunNoErrors(
  ctx: CheckContext,
  frames: number = 5,
  sceneRelPath: string = "scene.json"
): Promise<CheckResult> {
  const checkName = `headless run has no errors in ${frames} frames`;
  const scenePath = path.join(ctx.workspaceDir, sceneRelPath);

  try {
    await fs.access(scenePath);
  } catch {
    return {
      name: checkName,
      passed: false,
      message: `Scene file not found at ${scenePath}`,
    };
  }

  try {
    const { stdout, stderr } = await execFileAsync("cargo", [
      "run",
      "-p",
      "forge_runtime",
      "--",
      "--headless",
      "--frames",
      frames.toString(),
      "--scene",
      scenePath,
    ]);

    const combined = `${stdout}\n${stderr}`;
    if (combined.includes("ERROR") || combined.includes("panicked at")) {
      return {
        name: checkName,
        passed: false,
        message: `Headless runtime logged errors: ${combined.slice(0, 500)}`,
      };
    }

    return {
      name: checkName,
      passed: true,
      message: `Successfully executed ${frames} frames headless`,
    };
  } catch (error: unknown) {
    return {
      name: checkName,
      passed: false,
      message: `Headless runtime execution crashed: ${getErrorMessage(error)}`,
    };
  }
}

/**
 * 3. Checks that an entity or component exists in the scene.
 */
export async function checkEntityOrComponentExists(
  ctx: CheckContext,
  predicate: {
    entityName?: string;
    entityId?: string;
    componentType?: string;
    componentMatcher?: (data: Record<string, unknown>) => boolean;
  },
  sceneRelPath: string = "scene.json"
): Promise<CheckResult> {
  const checkName = `entity/component exists: ${
    predicate.entityName || predicate.entityId || predicate.componentType || "any"
  }`;
  const scenePath = path.join(ctx.workspaceDir, sceneRelPath);

  try {
    const raw = await fs.readFile(scenePath, "utf-8");
    const scene = JSON.parse(raw) as SceneJsonLike;

    const entities: SceneEntityLike[] = scene.entities || [];
    const matched = entities.filter((ent) => {
      if (predicate.entityId && ent.id !== predicate.entityId) return false;
      if (predicate.entityName && ent.name !== predicate.entityName) return false;

      if (predicate.componentType) {
        const comps = ent.components || {};
        if (!comps[predicate.componentType]) return false;
        if (
          predicate.componentMatcher &&
          !predicate.componentMatcher(comps[predicate.componentType])
        ) {
          return false;
        }
      }
      return true;
    });

    if (matched.length > 0) {
      return {
        name: checkName,
        passed: true,
        message: `Found ${matched.length} matching entity/component(s)`,
      };
    }

    return {
      name: checkName,
      passed: false,
      message: `No entity matched criteria: ${JSON.stringify(predicate)}`,
    };
  } catch (error: unknown) {
    return {
      name: checkName,
      passed: false,
      message: `Failed to inspect scene: ${getErrorMessage(error)}`,
    };
  }
}

/**
 * 4. Checks that scripted input or simulation changes game state in the expected way.
 */
export async function checkScriptedInputState(
  ctx: CheckContext,
  verifier: (workspaceDir: string) => Promise<{ passed: boolean; message: string }>
): Promise<CheckResult> {
  const checkName = "scripted input changes game state in expected way";
  try {
    const res = await verifier(ctx.workspaceDir);
    return {
      name: checkName,
      passed: res.passed,
      message: res.message,
    };
  } catch (err: unknown) {
    return {
      name: checkName,
      passed: false,
      message: `Failed checking game state: ${getErrorMessage(err)}`,
    };
  }
}

/**
 * 5. Checks that a tilemap tile equals an expected value at given coordinates.
 */
export async function checkTilemapTileEquals(
  ctx: CheckContext,
  layer: string,
  tileX: number,
  tileY: number,
  expectedValue: number | string,
  sceneRelPath: string = "scene.json"
): Promise<CheckResult> {
  const checkName = `tilemap layer '${layer}' at (${tileX}, ${tileY}) equals ${expectedValue}`;
  const scenePath = path.join(ctx.workspaceDir, sceneRelPath);

  try {
    const raw = await fs.readFile(scenePath, "utf-8");
    const scene = JSON.parse(raw) as {
      entities?: Array<{ components?: { Tilemap?: { chunk_size?: number; chunks?: Record<string, { layers?: Record<string, string[]> }> } } }>;
    };

    const tilemapEntity = (scene.entities || []).find(
      (ent) => ent.components && ent.components.Tilemap
    );

    if (!tilemapEntity || !tilemapEntity.components?.Tilemap) {
      return {
        name: checkName,
        passed: false,
        message: "No entity with Tilemap component found in scene",
      };
    }

    const tilemap = tilemapEntity.components.Tilemap;
    const chunkSize = tilemap.chunk_size || 16;
    const chunkX = Math.floor(tileX / chunkSize);
    const chunkY = Math.floor(tileY / chunkSize);
    const localX = ((tileX % chunkSize) + chunkSize) % chunkSize;
    const localY = ((tileY % chunkSize) + chunkSize) % chunkSize;

    const chunkKey = `[${chunkX}, ${chunkY}]`;
    const chunk = tilemap.chunks?.[chunkKey];

    if (!chunk || !chunk.layers || !chunk.layers[layer]) {
      return {
        name: checkName,
        passed: false,
        message: `Chunk ${chunkKey} or layer '${layer}' not found in Tilemap`,
      };
    }

    const layerRows: string[] = chunk.layers[layer];
    if (localY >= layerRows.length) {
      return {
        name: checkName,
        passed: false,
        message: `Row ${localY} out of bounds in layer '${layer}'`,
      };
    }

    const rowTiles = layerRows[localY].split(",");
    const actualVal = rowTiles[localX]?.trim();

    if (String(actualVal) === String(expectedValue)) {
      return {
        name: checkName,
        passed: true,
        message: `Tile at (${tileX}, ${tileY}) matches expected value ${expectedValue}`,
      };
    }

    return {
      name: checkName,
      passed: false,
      message: `Expected tile ${expectedValue} at (${tileX}, ${tileY}), got '${actualVal}'`,
    };
  } catch (error: unknown) {
    return {
      name: checkName,
      passed: false,
      message: `Failed inspecting tilemap tile: ${getErrorMessage(error)}`,
    };
  }
}

/**
 * 6. Checks that a screenshot passes a vision judge with a rubric (optional and clearly marked as soft).
 */
export async function checkScreenshotVisionRubric(
  ctx: CheckContext,
  rubric: string,
  screenshotName: string = "screenshot.png",
  judgeFn?: (screenshotBuffer: Buffer, rubric: string) => Promise<{ passed: boolean; score: number; critique: string }>
): Promise<CheckResult> {
  const checkName = `[SOFT] Vision judge rubric: ${rubric.slice(0, 40)}...`;
  const screenshotPath = path.join(ctx.workspaceDir, screenshotName);

  try {
    let buf: Buffer;
    try {
      buf = await fs.readFile(screenshotPath);
    } catch {
      return {
        name: checkName,
        passed: true, // soft check does not fail the hard run if screenshot missing
        message: `Screenshot '${screenshotName}' not found; soft vision check skipped`,
        is_soft: true,
      };
    }

    if (!judgeFn) {
      return {
        name: checkName,
        passed: true,
        message: `Screenshot exists (${buf.length} bytes), judge not configured (soft check passed)`,
        is_soft: true,
      };
    }

    const verdict = await judgeFn(buf, rubric);
    return {
      name: checkName,
      passed: verdict.passed,
      message: `Vision judge (${verdict.score}/10): ${verdict.critique}`,
      is_soft: true,
    };
  } catch (err: unknown) {
    return {
      name: checkName,
      passed: true, // soft check
      message: `Vision judge soft warning: ${getErrorMessage(err)}`,
      is_soft: true,
    };
  }
}

/**
 * Helper to inspect script file contents directly.
 */
export async function checkFileContains(
  ctx: CheckContext,
  relativeFilePath: string,
  patternOrSubstring: string | RegExp
): Promise<CheckResult> {
  const checkName = `file '${relativeFilePath}' contains expected code/pattern`;
  const filePath = path.join(ctx.workspaceDir, relativeFilePath);

  try {
    const content = await fs.readFile(filePath, "utf-8");
    const matched =
      typeof patternOrSubstring === "string"
        ? content.includes(patternOrSubstring)
        : patternOrSubstring.test(content);

    if (matched) {
      return {
        name: checkName,
        passed: true,
        message: `File '${relativeFilePath}' matched pattern`,
      };
    }
    return {
      name: checkName,
      passed: false,
      message: `File '${relativeFilePath}' did not contain expected content`,
    };
  } catch (err: unknown) {
    return {
      name: checkName,
      passed: false,
      message: `Could not read file '${filePath}': ${getErrorMessage(err)}`,
    };
  }
}
