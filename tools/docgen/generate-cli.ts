import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, "../..");
const CLI_REF_DIR = path.join(REPO_ROOT, "docs/src/reference/cli");

fs.mkdirSync(CLI_REF_DIR, { recursive: true });

export interface CliCommandDef {
  name: string;
  syntax: string;
  purpose: string;
  arguments: Array<{ flag: string; type: string; required: boolean; defaultVal?: string; description: string }>;
  examples: Array<{ command: string; explanation: string }>;
}

export const CLI_COMMANDS: CliCommandDef[] = [
  {
    name: "call",
    syntax: "forge call <METHOD> [PARAMS] [OPTIONS]",
    purpose: "Connects to a running Forge runtime instance and invokes a JSON-RPC 2.0 or MCP method.",
    arguments: [
      { flag: "<METHOD>", type: "string", required: true, description: "Name of the method or MCP tool to call (e.g. engine.ping, scene.get_tree, tools/list)" },
      { flag: "[PARAMS]", type: "string", required: false, defaultVal: "'{}'", description: "JSON string payload arguments" },
      { flag: "-p, --port <PORT>", type: "number", required: false, description: "WebSocket port of running bridge server (reads .forge/bridge.json if omitted)" },
      { flag: "-t, --token <TOKEN>", type: "string", required: false, description: "Authentication bearer token (reads .forge/bridge.json if omitted)" }
    ],
    examples: [
      { command: "forge call engine.ping", explanation: "Sends ping heartbeat to verify engine responsiveness" },
      { command: 'forge call scene.create_entity \'{"name":"Hero"}\'', explanation: "Instantiates a new entity in the active scene" },
      { command: "forge call tools/list", explanation: "Queries all registered MCP tools" }
    ]
  },
  {
    name: "run",
    syntax: "forge run [SCENE] [OPTIONS]",
    purpose: "Launches the standalone Forge game runtime process.",
    arguments: [
      { flag: "[SCENE]", type: "string", required: false, defaultVal: "templates/hello/scene.json", description: "Path to the initial scene JSON file to load" },
      { flag: "--headless", type: "boolean", required: false, defaultVal: "false", description: "Run offscreen headless rendering without an interactive window" },
      { flag: "--port <PORT>", type: "number", required: false, description: "Custom port to request for the IPC bridge server" }
    ],
    examples: [
      { command: "forge run templates/platformer/scene.json", explanation: "Launches platformer template in interactive window" },
      { command: "forge run --headless --port 9005", explanation: "Starts headless engine server on port 9005 for CI or agent test harness" }
    ]
  },
  {
    name: "check",
    syntax: "forge check [PROJECT] [OPTIONS]",
    purpose: "Runs TypeScript type checking on a project's scripts folder against forge.d.ts.",
    arguments: [
      { flag: "[PROJECT]", type: "string", required: false, defaultVal: "current dir", description: "Project directory containing scripts/ and tsconfig.json" },
      { flag: "-p, --project <DIR>", type: "string", required: false, description: "Alternative flag for project directory" },
      { flag: "--json", type: "boolean", required: false, defaultVal: "false", description: "Print diagnostics as structured JSON for autonomous agents" }
    ],
    examples: [
      { command: "forge check templates/platformer", explanation: "Type checks platformer scripts and prints human diagnostics" },
      { command: "forge check --json", explanation: "Returns structured JSON diagnostics for agent auto-repair loops" }
    ]
  },
  {
    name: "new",
    syntax: "forge new <NAME> [OPTIONS]",
    purpose: "Creates a new game project from a starter template and initializes its procedural assets.",
    arguments: [
      { flag: "<NAME>", type: "string", required: true, description: "Name of the target project directory to create" },
      { flag: "--template <TEMPLATE>", type: "string", required: false, defaultVal: "platformer", description: "Starter template to use ('platformer', 'hello', 'tilemap-demo', 'script-demo')" }
    ],
    examples: [
      { command: "forge new my-game --template platformer", explanation: "Creates a new 2D platformer starter project" }
    ]
  },
  {
    name: "index",
    syntax: "forge index [PROJECT]",
    purpose: "Generates .forge/map.json indexing scenes, entities, scripts, assets, clips, and input actions.",
    arguments: [
      { flag: "[PROJECT]", type: "string", required: false, defaultVal: ".", description: "Project directory to index" }
    ],
    examples: [
      { command: "forge index templates/platformer", explanation: "Builds deterministic .forge/map.json cache in under 200ms" }
    ]
  }
];

export function generateCli(): string[] {
  const generatedIds: string[] = [];

  for (const cmd of CLI_COMMANDS) {
    const id = `ref-cli-${cmd.name}`;
    generatedIds.push(id);

    const argsTable = cmd.arguments.map(a =>
      `| \`${a.flag}\` | \`${a.type}\` | ${a.required ? "**Yes**" : "No"} | \`${a.defaultVal || "-"}\` | ${a.description} |`
    ).join("\n");

    const examplesSection = cmd.examples.map(ex =>
      `**${ex.explanation}:**\n\`\`\`bash\n${ex.command}\n\`\`\``
    ).join("\n\n");

    const md = `---
id: ${id}
title: "CLI: forge ${cmd.name}"
summary: "${cmd.purpose}"
tags: ["reference", "cli", "${cmd.name}"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["ref-cli-index", "guide-getting-started"]
---

# \`forge ${cmd.name}\`

${cmd.purpose}

---

## Syntax

\`\`\`bash
${cmd.syntax}
\`\`\`

---

## Options & Arguments

| Option | Type | Required | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
${argsTable}

---

## Examples

${examplesSection}
`;
    fs.writeFileSync(path.join(CLI_REF_DIR, `${cmd.name}.md`), md, "utf-8");
  }

  // CLI Index
  const indexId = "ref-cli-index";
  generatedIds.push(indexId);
  const rows = CLI_COMMANDS.map(c =>
    `| [\`forge ${c.name}\`](file:///docs/src/reference/cli/${c.name}.md) | \`${c.syntax}\` | ${c.purpose} |`
  ).join("\n");

  const indexMd = `---
id: ${indexId}
title: "Forge CLI Command Reference"
summary: "Developer command line interface commands for running, validating, inspecting, and scaffolding Forge projects."
tags: ["reference", "cli", "index"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["ref-tools-index", "guide-getting-started"]
---

# Forge CLI Command Reference

The \`forge\` CLI provides direct terminal tools for project verification, runtime hosting, IPC queries, and project indexing:

| Command | Syntax | Purpose |
| :--- | :--- | :--- |
${rows}
`;

  fs.writeFileSync(path.join(CLI_REF_DIR, "index.md"), indexMd, "utf-8");

  return generatedIds;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  generateCli();
  console.log(`Generated ${CLI_COMMANDS.length} CLI command reference pages.`);
}
