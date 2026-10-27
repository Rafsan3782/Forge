import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, "../..");
const REF_DIR = path.join(REPO_ROOT, "docs/src/reference/components");

fs.mkdirSync(REF_DIR, { recursive: true });

export interface ComponentFieldDef {
  name: string;
  type: string;
  defaultValue: string;
  range?: string;
  widget: string;
  editorHint?: string;
  description: string;
}

export interface ComponentDef {
  name: string;
  displayName: string;
  category: string;
  purpose: string;
  fields: ComponentFieldDef[];
  exampleJson: string;
}

export const COMPONENTS: ComponentDef[] = [
  {
    name: "Name",
    displayName: "Name",
    category: "Core",
    purpose: "Provides a human-readable display label and identifier for an entity in the hierarchy.",
    fields: [
      {
        name: "name",
        type: "string",
        defaultValue: '"Entity"',
        widget: "text",
        description: "Display label displayed in the Studio Scene Tree."
      }
    ],
    exampleJson: '{\n  "Name": {\n    "name": "Hero"\n  }\n}'
  },
  {
    name: "Transform2D",
    displayName: "Transform 2D",
    category: "Spatial",
    purpose: "Defines 2D Cartesian spatial coordinates, heading angle in radians, and scale factors.",
    fields: [
      {
        name: "position",
        type: "[number, number]",
        defaultValue: "[0.0, 0.0]",
        widget: "vector2",
        editorHint: "step: 0.5",
        description: "World coordinate position [x, y] in pixels."
      },
      {
        name: "rotation",
        type: "number",
        defaultValue: "0.0",
        range: "-360.0 to 360.0 degrees (stored in radians internally)",
        widget: "number",
        editorHint: "min: -360.0, max: 360.0, step: 1.0",
        description: "Rotation angle around the Z axis in radians."
      },
      {
        name: "scale",
        type: "[number, number]",
        defaultValue: "[1.0, 1.0]",
        range: "0.01 to 100.0",
        widget: "vector2",
        editorHint: "min: 0.01, max: 100.0, step: 0.1",
        description: "Non-uniform horizontal and vertical scale multipliers."
      }
    ],
    exampleJson: '{\n  "Transform2D": {\n    "position": [120.0, 64.0],\n    "rotation": 0.0,\n    "scale": [1.0, 1.0]\n  }\n}'
  },
  {
    name: "Sprite",
    displayName: "Sprite",
    category: "2D Graphics",
    purpose: "Renders a 2D textured quad with size, color tint, and z-sorting order.",
    fields: [
      {
        name: "asset_id",
        type: "string",
        defaultValue: '""',
        widget: "asset",
        editorHint: "asset_type: 'texture'",
        description: "UUID or relative path of the texture asset."
      },
      {
        name: "size",
        type: "[number, number]",
        defaultValue: "[32.0, 32.0]",
        range: "1.0 to 2048.0",
        widget: "vector2",
        editorHint: "min: 1.0, max: 2048.0, step: 1.0",
        description: "Rendered dimensions in world coordinates [width, height]."
      },
      {
        name: "color_tint",
        type: "[number, number, number, number]",
        defaultValue: "[1.0, 1.0, 1.0, 1.0]",
        widget: "color",
        description: "Normalized RGBA color tint multiplier."
      },
      {
        name: "z_order",
        type: "number",
        defaultValue: "0",
        range: "-100.0 to 100.0",
        widget: "number",
        editorHint: "min: -100.0, max: 100.0, step: 1.0",
        description: "Layer rendering priority for sprite batching."
      }
    ],
    exampleJson: '{\n  "Sprite": {\n    "asset_id": "assets/player.png",\n    "size": [32.0, 32.0],\n    "color_tint": [1.0, 1.0, 1.0, 1.0],\n    "z_order": 5\n  }\n}'
  },
  {
    name: "Camera2D",
    displayName: "Camera 2D",
    category: "Graphics",
    purpose: "Defines the 2D orthographic viewport projection and active camera zoom level.",
    fields: [
      {
        name: "zoom",
        type: "number",
        defaultValue: "1.0",
        range: "0.1 to 10.0",
        widget: "number",
        editorHint: "min: 0.1, max: 10.0, step: 0.1",
        description: "Magnification factor for rendering the scene."
      },
      {
        name: "is_active",
        type: "boolean",
        defaultValue: "true",
        widget: "bool",
        description: "Whether this camera currently directs main viewport rendering."
      }
    ],
    exampleJson: '{\n  "Camera2D": {\n    "zoom": 1.5,\n    "is_active": true\n  }\n}'
  },
  {
    name: "RigidBody2D",
    displayName: "Rigid Body 2D",
    category: "Physics",
    purpose: "Integrates entity into the Rapier 2D physics simulation with linear/angular dynamics.",
    fields: [
      {
        name: "body_type",
        type: '"dynamic" | "fixed" | "kinematic"',
        defaultValue: '"dynamic"',
        widget: "enum",
        editorHint: "options: ['dynamic', 'fixed', 'kinematic']",
        description: "Body simulation dynamics type."
      },
      {
        name: "gravity_scale",
        type: "number",
        defaultValue: "1.0",
        range: "-10.0 to 10.0",
        widget: "number",
        editorHint: "min: -10.0, max: 10.0, step: 0.1",
        description: "Multiplier applied to world gravity vector."
      },
      {
        name: "linear_damping",
        type: "number",
        defaultValue: "0.0",
        range: "0.0 to 10.0",
        widget: "number",
        editorHint: "min: 0.0, max: 10.0, step: 0.1",
        description: "Velocity reduction rate per second."
      }
    ],
    exampleJson: '{\n  "RigidBody2D": {\n    "body_type": "dynamic",\n    "gravity_scale": 1.0,\n    "linear_damping": 0.1\n  }\n}'
  },
  {
    name: "Collider2D",
    displayName: "Collider 2D",
    category: "Physics",
    purpose: "Defines geometry bounding volume for physical collision and trigger overlaps.",
    fields: [
      {
        name: "shape",
        type: '"box" | "circle" | "capsule"',
        defaultValue: '"box"',
        widget: "enum",
        editorHint: "options: ['box', 'circle']",
        description: "Collision bounding geometry type."
      },
      {
        name: "size",
        type: "[number, number]",
        defaultValue: "[32.0, 32.0]",
        range: "1.0 to 1000.0",
        widget: "vector2",
        editorHint: "min: 1.0, max: 1000.0, step: 1.0",
        description: "Half-extents [hx, hy] for box, or [radius, 0] for circle."
      },
      {
        name: "is_sensor",
        type: "boolean",
        defaultValue: "false",
        widget: "bool",
        description: "If true, registers collision events without exerting physical impulses."
      },
      {
        name: "friction",
        type: "number",
        defaultValue: "0.5",
        range: "0.0 to 2.0",
        widget: "number",
        editorHint: "min: 0.0, max: 2.0, step: 0.05",
        description: "Coulomb surface friction coefficient."
      }
    ],
    exampleJson: '{\n  "Collider2D": {\n    "shape": "box",\n    "size": [16.0, 16.0],\n    "is_sensor": false,\n    "friction": 0.4\n  }\n}'
  },
  {
    name: "Script",
    displayName: "Script",
    category: "Logic",
    purpose: "Attaches a TypeScript behavior script file to an entity for custom logic.",
    fields: [
      {
        name: "path",
        type: "string",
        defaultValue: '""',
        widget: "asset",
        editorHint: "asset_type: 'script'",
        description: "Project-relative path to TypeScript script file (e.g. 'scripts/player.ts')."
      },
      {
        name: "enabled",
        type: "boolean",
        defaultValue: "true",
        widget: "bool",
        description: "Whether the script lifecycle functions are actively executed."
      }
    ],
    exampleJson: '{\n  "Script": {\n    "path": "scripts/player.ts",\n    "enabled": true\n  }\n}'
  },
  {
    name: "Tilemap",
    displayName: "Tilemap",
    category: "2D Graphics",
    purpose: "Renders chunked 2D grid-aligned tile graphics using a shared tileset texture.",
    fields: [
      {
        name: "tileset_id",
        type: "string",
        defaultValue: '""',
        widget: "asset",
        editorHint: "asset_type: 'tileset'",
        description: "Asset ID or path of tileset descriptor."
      },
      {
        name: "tile_size",
        type: "[number, number]",
        defaultValue: "[16.0, 16.0]",
        range: "4.0 to 256.0",
        widget: "vector2",
        editorHint: "min: 4.0, max: 256.0, step: 1.0",
        description: "Dimensions of individual tiles in pixels."
      },
      {
        name: "chunk_size",
        type: "number",
        defaultValue: "16",
        range: "8.0 to 64.0",
        widget: "number",
        editorHint: "min: 8.0, max: 64.0, step: 8.0",
        description: "Tile count per edge of each chunk."
      }
    ],
    exampleJson: '{\n  "Tilemap": {\n    "tileset_id": "assets/tileset.png",\n    "tile_size": [16.0, 16.0],\n    "chunk_size": 16\n  }\n}'
  }
];

export function generateComponents(): string[] {
  const generatedIds: string[] = [];

  for (const comp of COMPONENTS) {
    const id = `ref-comp-${comp.name.toLowerCase()}`;
    generatedIds.push(id);

    const fieldsTable = comp.fields.map(f =>
      `| \`${f.name}\` | \`${f.type}\` | \`${f.defaultValue}\` | ${f.range || "-"} | \`${f.widget}\` | ${f.editorHint || "-"} | ${f.description} |`
    ).join("\n");

    const md = `---
id: ${id}
title: "Component: ${comp.name}"
summary: "${comp.purpose}"
tags: ["reference", "component", "${comp.category.toLowerCase()}"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["ref-components-index", "concept-ecs-architecture", "ref-tool-scene-set_component"]
---

# Component: \`${comp.name}\`

**Display Name:** ${comp.displayName}  
**Category:** \`${comp.category}\`  
**Purpose:** ${comp.purpose}

---

## Fields & Editor Hints

| Field | Type | Default | Range | Widget | Editor Hint | Description |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
${fieldsTable}

---

## JSON Representation

\`\`\`json
${comp.exampleJson}
\`\`\`

---

## Script API Access

\`\`\`snippet
// Example accessing ${comp.name} from an entity script
console.log("Component ${comp.name}");
\`\`\`
`;

    fs.writeFileSync(path.join(REF_DIR, `${comp.name}.md`), md, "utf-8");
  }

  // Component index
  const indexId = "ref-components-index";
  generatedIds.push(indexId);
  const indexRows = COMPONENTS.map(c =>
    `| [\`${c.name}\`](file:///docs/src/reference/components/${c.name}.md) | \`${c.category}\` | ${c.purpose} |`
  ).join("\n");

  const indexMd = `---
id: ${indexId}
title: "Built-in ECS Components Index"
summary: "Catalog of all standard ECS components available in Forge scenes and scripts."
tags: ["reference", "components", "index"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["ref-tools-index", "concept-ecs-architecture"]
---

# Built-in ECS Components Index

Forge uses a high-performance entity component architecture with dynamic schemas for Studio Inspector binding.

| Component | Category | Purpose |
| :--- | :--- | :--- |
${indexRows}
`;

  fs.writeFileSync(path.join(REF_DIR, "index.md"), indexMd, "utf-8");

  return generatedIds;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  generateComponents();
  console.log(`Generated ${COMPONENTS.length} component reference pages.`);
}
