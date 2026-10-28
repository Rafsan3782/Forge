import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, "../..");
const SCRIPT_REF_DIR = path.join(REPO_ROOT, "docs/src/reference/script-api");

fs.mkdirSync(SCRIPT_REF_DIR, { recursive: true });

export interface ScriptApiMethodDef {
  module: string;
  name: string;
  signature: string;
  description: string;
  example: string;
}

export const SCRIPT_MODULES: Array<{
  module: string;
  description: string;
  methods: ScriptApiMethodDef[];
}> = [
  {
    module: "forge.entity",
    description: "Inspect and manipulate the host entity executing the current script.",
    methods: [
      {
        module: "forge.entity",
        name: "getId",
        signature: "forge.entity.getId(): string",
        description: "Returns the stable UUID identifier of the current entity.",
        example: 'const myId = forge.entity.getId();'
      },
      {
        module: "forge.entity",
        name: "getName",
        signature: "forge.entity.getName(): string",
        description: "Returns the display name string of the current entity.",
        example: 'forge.log.info("Entity started: " + forge.entity.getName());'
      },
      {
        module: "forge.entity",
        name: "getParent",
        signature: "forge.entity.getParent(): string | null",
        description: "Returns the entity ID of this entity\'s parent node, or null if root.",
        example: 'const parentId = forge.entity.getParent();'
      },
      {
        module: "forge.entity",
        name: "getChildren",
        signature: "forge.entity.getChildren(): string[]",
        description: "Returns an array of entity IDs belonging to direct descendants.",
        example: 'const children = forge.entity.getChildren();'
      },
      {
        module: "forge.entity",
        name: "getTransform",
        signature: "forge.entity.getTransform(): ScriptEntityTransform",
        description: "Returns the current position [x, y], rotation (rad), and scale [sx, sy].",
        example: 'const { position, rotation, scale } = forge.entity.getTransform();'
      },
      {
        module: "forge.entity",
        name: "setTransform",
        signature: "forge.entity.setTransform(transform: Partial<ScriptEntityTransform>): void",
        description: "Applies position, rotation, or scale updates back to the entity in ECS.",
        example: 'forge.entity.setTransform({ position: [100.0, 50.0] });'
      },
      {
        module: "forge.entity",
        name: "getComponent",
        signature: "forge.entity.getComponent<T = unknown>(name: string): T | null",
        description: "Retrieves serialized properties of an attached component by name.",
        example: 'const sprite = forge.entity.getComponent<{ asset_id: string }>("Sprite");'
      },
      {
        module: "forge.entity",
        name: "setComponent",
        signature: "forge.entity.setComponent<T = unknown>(name: string, data: T): void",
        description: "Updates or adds a component on the entity.",
        example: 'forge.entity.setComponent("Sprite", { asset_id: "assets/player.png" });'
      }
    ]
  },
  {
    module: "forge.input",
    description: "Query keyboard, mouse, and mapped action input states.",
    methods: [
      {
        module: "forge.input",
        name: "isActionPressed",
        signature: "forge.input.isActionPressed(action: string): boolean",
        description: "Returns true if the action is currently held down this frame.",
        example: 'if (forge.input.isActionPressed("move_left")) { /* move */ }'
      },
      {
        module: "forge.input",
        name: "isActionJustPressed",
        signature: "forge.input.isActionJustPressed(action: string): boolean",
        description: "Returns true only on the frame the action was initially pressed.",
        example: 'if (forge.input.isActionJustPressed("jump")) { /* jump */ }'
      },
      {
        module: "forge.input",
        name: "isActionJustReleased",
        signature: "forge.input.isActionJustReleased(action: string): boolean",
        description: "Returns true only on the frame the action was released.",
        example: 'if (forge.input.isActionJustReleased("jump")) { /* cut jump */ }'
      },
      {
        module: "forge.input",
        name: "getAxis",
        signature: "forge.input.getAxis(axis: string): number",
        description: "Returns normalized analog or digital axis value (-1.0 to 1.0).",
        example: 'const moveX = forge.input.getAxis("horizontal");'
      },
      {
        module: "forge.input",
        name: "getMousePosition",
        signature: "forge.input.getMousePosition(): [number, number]",
        description: "Returns current mouse cursor coordinates [x, y] in screen space.",
        example: 'const [mx, my] = forge.input.getMousePosition();'
      }
    ]
  },
  {
    module: "forge.physics",
    description: "2D spatial queries and raycasting against the Rapier physics world.",
    methods: [
      {
        module: "forge.physics",
        name: "raycast",
        signature: "forge.physics.raycast(origin: [number, number], direction: [number, number], maxDistance: number): ScriptRaycastHit | null",
        description: "Casts a ray and returns nearest collider hit or null if none.",
        example: 'const hit = forge.physics.raycast([0, 0], [0, -1], 100);'
      },
      {
        module: "forge.physics",
        name: "overlapCircle",
        signature: "forge.physics.overlapCircle(center: [number, number], radius: number): string[]",
        description: "Queries all entity IDs whose colliders overlap a circle.",
        example: 'const nearbyEntities = forge.physics.overlapCircle([100, 50], 32.0);'
      }
    ]
  },
  {
    module: "forge.time",
    description: "Frame timing, delta time, and elapsed play session metrics.",
    methods: [
      {
        module: "forge.time",
        name: "deltaTime",
        signature: "forge.time.deltaTime(): number",
        description: "Returns time elapsed since previous frame in seconds.",
        example: 'const dt = forge.time.deltaTime();'
      },
      {
        module: "forge.time",
        name: "elapsedTime",
        signature: "forge.time.elapsedTime(): number",
        description: "Returns total seconds elapsed since play mode started.",
        example: 'const totalSecs = forge.time.elapsedTime();'
      },
      {
        module: "forge.time",
        name: "frameCount",
        signature: "forge.time.frameCount(): number",
        description: "Returns total simulation frames stepped so far.",
        example: 'const frame = forge.time.frameCount();'
      }
    ]
  },
  {
    module: "forge.audio",
    description: "Trigger sound effects and music tracks on audio buses.",
    methods: [
      {
        module: "forge.audio",
        name: "play",
        signature: 'forge.audio.play(assetId: string, options?: { bus?: "master" | "music" | "sfx"; volume?: number; pitch?: number; loop?: boolean }): string',
        description: "Plays audio asset and returns unique playback instance ID.",
        example: 'forge.audio.play("assets/coin.wav", { bus: "sfx", volume: 0.8 });'
      },
      {
        module: "forge.audio",
        name: "stop",
        signature: 'forge.audio.stop(playbackId?: string, bus?: "master" | "music" | "sfx"): void',
        description: "Stops a specific playback ID or all audio playing on a bus channel.",
        example: 'forge.audio.stop(undefined, "music");'
      },
      {
        module: "forge.audio",
        name: "setBusVolume",
        signature: 'forge.audio.setBusVolume(bus: "master" | "music" | "sfx", volume: number): void',
        description: "Updates volume level (0.0 to 1.0) on an audio bus.",
        example: 'forge.audio.setBusVolume("music", 0.5);'
      }
    ]
  },
  {
    module: "forge.tilemap",
    description: "Direct procedural read and write access to tilemap grid chunks.",
    methods: [
      {
        module: "forge.tilemap",
        name: "getTile",
        signature: "forge.tilemap.getTile(entityId: string, layer: string, x: number, y: number): number",
        description: "Returns tile index at grid coordinate (0 for empty).",
        example: 'const tile = forge.tilemap.getTile(mapId, "main", 5, 10);'
      },
      {
        module: "forge.tilemap",
        name: "setTile",
        signature: "forge.tilemap.setTile(entityId: string, layer: string, x: number, y: number, tileId: number): void",
        description: "Sets or erases tile index at grid coordinate.",
        example: 'forge.tilemap.setTile(mapId, "main", 5, 10, 1);'
      }
    ]
  },
  {
    module: "forge.log",
    description: "Structured console output streamed back to the Studio IDE and agent telemetry.",
    methods: [
      {
        module: "forge.log",
        name: "info",
        signature: "forge.log.info(...args: unknown[]): void",
        description: "Logs an informational message to engine log stream.",
        example: 'forge.log.info("Player spawned at", [x, y]);'
      },
      {
        module: "forge.log",
        name: "warn",
        signature: "forge.log.warn(...args: unknown[]): void",
        description: "Logs a warning message to engine log stream.",
        example: 'forge.log.warn("Low health:", health);'
      },
      {
        module: "forge.log",
        name: "error",
        signature: "forge.log.error(...args: unknown[]): void",
        description: "Logs an error message to engine log stream.",
        example: 'forge.log.error("Failed to load scene:", err);'
      }
    ]
  }
];

export function generateScriptApi(): string[] {
  const generatedIds: string[] = [];

  for (const mod of SCRIPT_MODULES) {
    const safeModName = mod.module.replace(/[^a-z0-9]/gi, "-");
    const id = `ref-script-${safeModName}`;
    generatedIds.push(id);

    const methodsDoc = mod.methods.map(m => `
### \`${m.name}\`

\`\`\`ts
${m.signature}
\`\`\`

${m.description}

**Example:**
\`\`\`ts
${m.example}
\`\`\`
`).join("\n---\n");

    const md = `---
id: ${id}
title: "Script API: ${mod.module}"
summary: "${mod.description}"
tags: ["reference", "script-api", "${safeModName}"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["ref-script-index", "guide-scripting-basics", "cookbook-player-controller"]
---

# Script API: \`${mod.module}\`

${mod.description}

---

## Methods
${methodsDoc}

---

## Usage Snippet

\`\`\`snippet
// Example using ${mod.module}
console.log("Script module ${mod.module}");
\`\`\`
`;
    fs.writeFileSync(path.join(SCRIPT_REF_DIR, `${safeModName}.md`), md, "utf-8");
  }

  // Script API Index
  const indexId = "ref-script-index";
  generatedIds.push(indexId);
  const rows = SCRIPT_MODULES.map(m =>
    `| [\`${m.module}\`](file:///docs/src/reference/script-api/${m.module.replace(/[^a-z0-9]/gi, "-")}.md) | ${m.methods.length} | ${m.description} |`
  ).join("\n");

  const indexMd = `---
id: ${indexId}
title: "Forge Script API Reference Index"
summary: "Complete API reference for global forge.* APIs and lifecycle callbacks available in TypeScript game scripts."
tags: ["reference", "script-api", "index"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["ref-tools-index", "guide-scripting-basics"]
---

# Forge Script API Reference Index

All Forge gameplay scripts run inside embedded QuickJS with type safety powered by \`forge.d.ts\`. The global \`forge\` object provides direct access to the engine:

| Module | Methods | Description |
| :--- | :--- | :--- |
${rows}

---

## Lifecycle Callbacks

Scripts can export the following optional lifecycle functions:

- \`onStart(): void\` - Invoked when the entity spawns or when play mode begins.
- \`onUpdate(dt: number): void\` - Invoked every frame before rendering with elapsed delta time in seconds.
- \`onFixedUpdate(dt: number): void\` - Invoked on constant-interval physics steps (60Hz).
- \`onCollisionEnter(otherEntityId: string): void\` - Triggered when a physical collider intersects another.
- \`onTriggerEnter(otherEntityId: string): void\` - Triggered when entering a sensor trigger zone.
- \`onDestroy(): void\` - Invoked when the entity is removed from the world.
`;

  fs.writeFileSync(path.join(SCRIPT_REF_DIR, "index.md"), indexMd, "utf-8");

  return generatedIds;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  generateScriptApi();
  console.log(`Generated ${SCRIPT_MODULES.length} script API modules.`);
}
