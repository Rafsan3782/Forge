import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, "../..");
const EVENTS_DIR = path.join(REPO_ROOT, "docs/src/reference/events");
const ERRORS_DIR = path.join(REPO_ROOT, "docs/src/reference/errors");

fs.mkdirSync(EVENTS_DIR, { recursive: true });
fs.mkdirSync(ERRORS_DIR, { recursive: true });

export interface EventDef {
  name: string;
  source: "Engine Runtime" | "Agent Orchestrator" | "Script System";
  description: string;
  payloadType: string;
  fields: Array<{ name: string; type: string; description: string }>;
  exampleJson: string;
}

export const EVENTS: EventDef[] = [
  {
    name: "game.log",
    source: "Engine Runtime",
    description: "Streamed log message emitted by engine subsystems or user scripts.",
    payloadType: "GameLogNotification",
    fields: [
      { name: "level", type: '"trace" | "debug" | "info" | "warn" | "error"', description: "Severity level of log event." },
      { name: "message", type: "string", description: "Log text payload." },
      { name: "source_location", type: "string", description: "Source file and line identifier where log originated." },
      { name: "frame", type: "number", description: "Simulation frame index when log occurred." }
    ],
    exampleJson: '{\n  "jsonrpc": "2.0",\n  "method": "game.log",\n  "params": {\n    "level": "info",\n    "message": "Player reached checkpoint 1",\n    "source_location": "scripts/player.ts:42",\n    "frame": 360\n  }\n}'
  },
  {
    name: "game.error",
    source: "Script System",
    description: "Runtime exception or fatal script error caught during update loop execution.",
    payloadType: "GameErrorNotification",
    fields: [
      { name: "level", type: '"error"', description: "Always 'error'." },
      { name: "message", type: "string", description: "Error description message." },
      { name: "source_location", type: "string", description: "Mapped TypeScript source file and line." },
      { name: "frame", type: "number", description: "Simulation frame counter." }
    ],
    exampleJson: '{\n  "jsonrpc": "2.0",\n  "method": "game.error",\n  "params": {\n    "level": "error",\n    "message": "TypeError: Cannot read properties of null",\n    "source_location": "scripts/patroller.ts:18",\n    "frame": 128\n  }\n}'
  },
  {
    name: "agent.run_started",
    source: "Agent Orchestrator",
    description: "Notification fired when an autonomous agent task run begins.",
    payloadType: "AgentEvent",
    fields: [
      { name: "type", type: '"run_started"', description: "Event type identifier." },
      { name: "run_id", type: "string", description: "Unique task execution run UUID." },
      { name: "timestamp", type: "number", description: "Epoch millisecond timestamp." },
      { name: "data", type: "object", description: "Run initialization parameters and prompt." }
    ],
    exampleJson: '{\n  "type": "run_started",\n  "run_id": "run-001",\n  "timestamp": 1710000000000,\n  "data": { "prompt": "Add jump mechanic" }\n}'
  },
  {
    name: "agent.plan_updated",
    source: "Agent Orchestrator",
    description: "Notification emitted when agent generates or updates its multi-step execution plan.",
    payloadType: "AgentEvent",
    fields: [
      { name: "type", type: '"plan_updated"', description: "Event type identifier." },
      { name: "run_id", type: "string", description: "Run identifier." },
      { name: "timestamp", type: "number", description: "Timestamp." },
      { name: "data", type: "Plan", description: "Updated plan with steps and acceptance criteria." }
    ],
    exampleJson: '{\n  "type": "plan_updated",\n  "run_id": "run-001",\n  "timestamp": 1710000001000,\n  "data": {\n    "id": "plan-1",\n    "title": "Implement jump",\n    "steps": [\n      { "id": "s1", "description": "Update player.ts", "status": "completed" }\n    ]\n  }\n}'
  },
  {
    name: "agent.tool_call_started",
    source: "Agent Orchestrator",
    description: "Broadcast when agent initiates execution of an MCP or bridge tool.",
    payloadType: "AgentEvent",
    fields: [
      { name: "type", type: '"tool_call_started"', description: "Event identifier." },
      { name: "run_id", type: "string", description: "Run ID." },
      { name: "timestamp", type: "number", description: "Timestamp." },
      { name: "data", type: "object", description: "Tool name and arguments payload." }
    ],
    exampleJson: '{\n  "type": "tool_call_started",\n  "run_id": "run-001",\n  "timestamp": 1710000002000,\n  "data": {\n    "tool": "script.check",\n    "args": { "project_path": "." }\n  }\n}'
  },
  {
    name: "agent.tool_call_finished",
    source: "Agent Orchestrator",
    description: "Broadcast when tool execution completes with return data or error.",
    payloadType: "AgentEvent",
    fields: [
      { name: "type", type: '"tool_call_finished"', description: "Event identifier." },
      { name: "run_id", type: "string", description: "Run ID." },
      { name: "timestamp", type: "number", description: "Timestamp." },
      { name: "data", type: "object", description: "Tool execution result and duration." }
    ],
    exampleJson: '{\n  "type": "tool_call_finished",\n  "run_id": "run-001",\n  "timestamp": 1710000003000,\n  "data": {\n    "tool": "script.check",\n    "success": true,\n    "duration_ms": 142\n  }\n}'
  }
];

export interface ErrorCodeDef {
  code: string | number;
  meaning: string;
  source: "JSON-RPC / Bridge" | "Agent Tool" | "TypeScript Runtime";
  commonCause: string;
  fix: string;
}

export const ERROR_CATALOG: ErrorCodeDef[] = [
  {
    code: -32700,
    meaning: "Parse Error",
    source: "JSON-RPC / Bridge",
    commonCause: "Invalid JSON payload sent over WebSocket connection.",
    fix: "Ensure message envelope is valid UTF-8 JSON matching JsonRpcRequest schema."
  },
  {
    code: -32600,
    meaning: "Invalid Request",
    source: "JSON-RPC / Bridge",
    commonCause: "The JSON-RPC request structure is missing jsonrpc version '2.0' or required id/method fields.",
    fix: "Verify payload conforms to JsonRpcRequest schema."
  },
  {
    code: -32601,
    meaning: "Method Not Found",
    source: "JSON-RPC / Bridge",
    commonCause: "The requested bridge method or tool name does not exist in the registry.",
    fix: "Consult the Bridge Tools Reference to verify exact tool method naming (e.g. 'scene.get_tree')."
  },
  {
    code: -32602,
    meaning: "Invalid Params",
    source: "JSON-RPC / Bridge",
    commonCause: "One or more required arguments are missing, mistyped, or out of range.",
    fix: "Inspect inputSchema for the tool and supply arguments matching types exactly."
  },
  {
    code: -32603,
    meaning: "Internal Error",
    source: "JSON-RPC / Bridge",
    commonCause: "Unhandled exception encountered inside bridge server dispatcher.",
    fix: "Review engine console output or restart the game runtime."
  },
  {
    code: -32000,
    meaning: "Engine Execution Error",
    source: "JSON-RPC / Bridge",
    commonCause: "The target entity was not found, asset failed to decode, or operation violated scene invariants.",
    fix: "Check entity IDs and asset paths before invoking mutation commands."
  },
  {
    code: -32001,
    meaning: "Unauthorized",
    source: "JSON-RPC / Bridge",
    commonCause: "Missing or incorrect security token in WebSocket connection query parameter (?token=...).",
    fix: "Read the active bridge connection credentials from .forge/bridge.json and supply matching token."
  },
  {
    code: -32002,
    meaning: "Channel Closed",
    source: "JSON-RPC / Bridge",
    commonCause: "The engine runtime process crashed or shut down while command was in flight.",
    fix: "Relaunch engine runtime via project.run or CLI forge run."
  },
  {
    code: "TOOL_ERROR",
    meaning: "Tool Execution Error",
    source: "Agent Tool",
    commonCause: "An agent tool encountered a validation or operational failure during execution.",
    fix: "Read the suggested_action field on the structured tool error and modify arguments."
  },
  {
    code: "PERMISSION_DENIED",
    meaning: "Agent Permission Denied",
    source: "Agent Tool",
    commonCause: "The requested tool action requires human confirmation under the active permission mode.",
    fix: "Prompt user for approval or adjust permission_mode in agent settings."
  },
  {
    code: "BUDGET_EXCEEDED",
    meaning: "Token or Turn Budget Exceeded",
    source: "Agent Tool",
    commonCause: "Agent loop exceeded maximum token limit or turn count specified in AgentSettings.",
    fix: "Increase max_tokens or summarize context before resuming execution."
  }
];

export function generateEventsAndErrors(): string[] {
  const generatedIds: string[] = [];

  // Generate Events
  for (const evt of EVENTS) {
    const id = `ref-event-${evt.name.replace(/[^a-z0-9]/gi, "-")}`;
    generatedIds.push(id);

    const fieldsTable = evt.fields.map(f =>
      `| \`${f.name}\` | \`${f.type}\` | ${f.description} |`
    ).join("\n");

    const md = `---
id: ${id}
title: "Event: ${evt.name}"
summary: "${evt.description}"
tags: ["reference", "event", "${evt.source.toLowerCase().replace(/[^a-z0-9]/g, "-")}"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["ref-events-index", "ref-tools-index"]
---

# Event: \`${evt.name}\`

**Source Subsystem:** \`${evt.source}\`  
**Payload Type:** \`${evt.payloadType}\`  
**Description:** ${evt.description}

---

## Event Fields

| Field | Type | Description |
| :--- | :--- | :--- |
${fieldsTable}

---

## Example Wire Payload

\`\`\`json
${evt.exampleJson}
\`\`\`
`;
    fs.writeFileSync(path.join(EVENTS_DIR, `${evt.name}.md`), md, "utf-8");
  }

  // Events Index
  const eventIndexId = "ref-events-index";
  generatedIds.push(eventIndexId);
  const eventRows = EVENTS.map(e =>
    `| [\`${e.name}\`](file:///docs/src/reference/events/${e.name}.md) | \`${e.source}\` | ${e.description} |`
  ).join("\n");

  const eventIndexMd = `---
id: ${eventIndexId}
title: "Events Reference Index"
summary: "Registry of all notification events streamed between engine, IDE, and AI agent runtimes."
tags: ["reference", "events", "index"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["ref-tools-index", "ref-errors-index"]
---

# Events Reference Index

| Event Name | Source | Description |
| :--- | :--- | :--- |
${eventRows}
`;
  fs.writeFileSync(path.join(EVENTS_DIR, "index.md"), eventIndexMd, "utf-8");

  // Generate Error Catalog
  const errorIndexId = "ref-errors-index";
  generatedIds.push(errorIndexId);
  const errorRows = ERROR_CATALOG.map(err =>
    `| \`${err.code}\` | **${err.meaning}** | \`${err.source}\` | ${err.commonCause} | ${err.fix} |`
  ).join("\n");

  const errorsMd = `---
id: ${errorIndexId}
title: "Error Catalog"
summary: "Comprehensive reference of error codes, common causes, and actionable fixes for Forge IPC and agent tooling."
tags: ["reference", "errors", "catalog"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["ref-tools-index", "ref-events-index"]
---

# Error Catalog

This catalog documents all standard JSON-RPC 2.0 error codes, bridge error details, and agent tool execution exceptions.

| Error Code | Meaning | Source | Common Cause | Recommended Fix |
| :--- | :--- | :--- | :--- | :--- |
${errorRows}

---

## Error Handling Pattern

\`\`\`snippet
// Example error check
console.log("Check error catalog");
\`\`\`
`;
  fs.writeFileSync(path.join(ERRORS_DIR, "index.md"), errorsMd, "utf-8");

  return generatedIds;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  generateEventsAndErrors();
  console.log(`Generated ${EVENTS.length} events and error catalog.`);
}
