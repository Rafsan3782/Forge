import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, "../..");
const REF_DIR = path.join(REPO_ROOT, "docs/src/reference");

fs.mkdirSync(path.join(REF_DIR, "bridge-tools"), { recursive: true });
fs.mkdirSync(path.join(REF_DIR, "components"), { recursive: true });
fs.mkdirSync(path.join(REF_DIR, "events"), { recursive: true });
fs.mkdirSync(path.join(REF_DIR, "errors"), { recursive: true });

interface BridgeToolDef {
  name: string;
  category: "Read" | "Write" | "Exec" | "Lifecycle" | "Sim" | "System";
  purpose: string;
  schemaFile?: string;
  paramsType?: string;
  resultType?: string;
  args: Array<{ name: string; type: string; required: boolean; description: string; example?: string }>;
  resultDesc: string;
  resultExample: string;
  errors: Array<{ code: number | string; meaning: string; commonCause: string; fix: string }>;
}

export const BRIDGE_TOOLS: BridgeToolDef[] = [
  {
    name: "engine.ping",
    category: "System",
    purpose: "Liveness probe and round-trip heartbeat check between host and engine runtime.",
    schemaFile: "engine.ping.json",
    paramsType: "EnginePingParams",
    resultType: "EnginePingResult",
    args: [
      { name: "timestamp", type: "number", required: false, description: "Epoch millisecond timestamp from client", example: "1710000000000" }
    ],
    resultDesc: "Object containing server status and engine echo timestamp.",
    resultExample: '{\n  "status": "ok",\n  "timestamp": 1710000000000\n}',
    errors: [
      { code: -32001, meaning: "Unauthorized", commonCause: "Missing or invalid bridge authentication token in handshake", fix: "Provide matching bearer token from .forge/bridge.json" },
      { code: -32002, meaning: "ChannelClosed", commonCause: "Engine runtime event loop terminated", fix: "Restart runtime or verify crash logs" }
    ]
  },
  {
    name: "scene.get_tree",
    category: "Read",
    purpose: "Query active hierarchy tree of entities, parent-child links, and component summaries.",
    schemaFile: "scene.get_tree.json",
    paramsType: "SceneGetTreeParams",
    resultType: "SceneGetTreeResult",
    args: [
      { name: "root_entity_id", type: "string | null", required: false, description: "Optional entity ID to restrict hierarchy query to a specific branch", example: '"entity_hero_01"' }
    ],
    resultDesc: "Full hierarchical scene node graph with root nodes and nested children.",
    resultExample: '{\n  "root_nodes": [\n    {\n      "id": "e_root",\n      "name": "WorldRoot",\n      "components": ["Transform2D"],\n      "children": []\n    }\n  ]\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Specified root entity does not exist in world", fix: "Query entire scene tree without root_entity_id filter" }
    ]
  },
  {
    name: "scene.create_entity",
    category: "Write",
    purpose: "Instantiate a new entity into the active scene with optional name, parent ID, and initial components.",
    schemaFile: "scene.create_entity.json",
    paramsType: "SceneCreateEntityParams",
    resultType: "SceneCreateEntityResult",
    args: [
      { name: "name", type: "string", required: true, description: "Descriptive name for the entity", example: '"Coin"' },
      { name: "parent_id", type: "string | null", required: false, description: "Parent entity ID in the hierarchy", example: '"entity_level_01"' },
      { name: "components", type: "Record<string, unknown> | null", required: false, description: "Map of component names to their initial fields", example: '{\n  "Transform2D": { "position": [100.0, 50.0] }\n}' }
    ],
    resultDesc: "Object containing the newly assigned stable UUID entity ID.",
    resultExample: '{\n  "entity_id": "c7a8b0e1-4567-4890-a123-abcdef123456"\n}',
    errors: [
      { code: -32602, meaning: "Invalid Params", commonCause: "Missing entity name or malformed component data", fix: "Verify entity name is non-empty string and component schema matches" }
    ]
  },
  {
    name: "scene.set_component",
    category: "Write",
    purpose: "Attach or mutate a component's serialized properties on a target entity.",
    schemaFile: "scene.set_component.json",
    paramsType: "SceneSetComponentParams",
    resultType: "SceneSetComponentResult",
    args: [
      { name: "entity_id", type: "string", required: true, description: "Target entity UUID", example: '"e_player_01"' },
      { name: "component_name", type: "string", required: true, description: "Name of component (e.g. Transform2D, Sprite, RigidBody2D)", example: '"Transform2D"' },
      { name: "data", type: "Record<string, unknown>", required: true, description: "Component property key-value map", example: '{\n  "position": [240.0, 180.0],\n  "rotation": 0.0,\n  "scale": [1.0, 1.0]\n}' }
    ],
    resultDesc: "Confirmation status object indicating success.",
    resultExample: '{\n  "success": true\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Entity not found or unknown component name", fix: "Check entity exists and component name is registered" }
    ]
  },
  {
    name: "scene.delete_entity",
    category: "Write",
    purpose: "Recursively delete an entity and all its child descendants from the ECS world.",
    schemaFile: "scene.delete_entity.json",
    paramsType: "SceneDeleteEntityParams",
    resultType: "SceneDeleteEntityResult",
    args: [
      { name: "entity_id", type: "string", required: true, description: "Entity UUID to remove", example: '"e_enemy_99"' }
    ],
    resultDesc: "Confirmation status object indicating success.",
    resultExample: '{\n  "success": true\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Entity ID not found", fix: "Check scene hierarchy before attempting deletion" }
    ]
  },
  {
    name: "game.screenshot",
    category: "Read",
    purpose: "Capture current viewport frame buffer offscreen as PNG/JPEG base64 data or save directly to disk.",
    schemaFile: "game.screenshot.json",
    paramsType: "GameScreenshotParams",
    resultType: "GameScreenshotResult",
    args: [
      { name: "format", type: '"png" | "jpeg"', required: false, description: "Image encoding format (default 'png')", example: '"png"' },
      { name: "width", type: "number | null", required: false, description: "Custom capture width in pixels", example: "800" },
      { name: "height", type: "number | null", required: false, description: "Custom capture height in pixels", example: "600" },
      { name: "file_path", type: "string | null", required: false, description: "Optional project-relative path to write image file", example: '"screenshots/frame_001.png"' }
    ],
    resultDesc: "Object containing base64 data URI string or saved file path confirmation.",
    resultExample: '{\n  "file_path": "screenshots/frame_001.png",\n  "width": 800,\n  "height": 600\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Graphics device context lost or invalid write path", fix: "Ensure parent directories exist for output file" }
    ]
  },
  {
    name: "game.simulate_input",
    category: "Exec",
    purpose: "Inject synthetic keyboard, mouse, and action events into the input queue for automated playtesting.",
    schemaFile: "game.simulate_input.json",
    paramsType: "GameSimulateInputParams",
    resultType: "GameSimulateInputResult",
    args: [
      { name: "events", type: "Array<SimulatedInputEvent>", required: true, description: "Batch list of input events to simulate", example: '[\n  { "input_type": "action_down", "action": "jump", "duration_frames": 2 }\n]' }
    ],
    resultDesc: "Status confirming number of events processed and frames queued.",
    resultExample: '{\n  "success": true,\n  "frames_simulated": 2\n}',
    errors: [
      { code: -32602, meaning: "Invalid Params", commonCause: "Unknown input_type or missing required action/key field", fix: "Review input_map.json and event schema definitions" }
    ]
  },
  {
    name: "project.run",
    category: "Lifecycle",
    purpose: "Launch or attach to a game runtime process for the current project.",
    schemaFile: "project.run.json",
    paramsType: "ProjectRunParams",
    resultType: "ProjectRunResult",
    args: [
      { name: "project_path", type: "string | null", required: false, description: "Path to project root directory", example: '"templates/platformer"' },
      { name: "scene", type: "string | null", required: false, description: "Initial scene file to launch", example: '"scene.json"' },
      { name: "headless", type: "boolean", required: false, description: "Whether to run without an interactive window", example: "true" }
    ],
    resultDesc: "Object with running process ID and bridge port.",
    resultExample: '{\n  "running": true,\n  "port": 9002\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Scene file not found or port already bound", fix: "Check scene path exists and check port availability" }
    ]
  },
  {
    name: "project.stop",
    category: "Lifecycle",
    purpose: "Terminate the running game runtime process gracefully.",
    schemaFile: "project.stop.json",
    paramsType: "ProjectStopParams",
    resultType: "ProjectStopResult",
    args: [
      { name: "session_id", type: "string | null", required: false, description: "Optional session ID to terminate", example: '""' }
    ],
    resultDesc: "Status confirming shutdown.",
    resultExample: '{\n  "stopped": true\n}',
    errors: [
      { code: -32002, meaning: "ChannelClosed", commonCause: "Process already terminated", fix: "Safe to ignore if process exited" }
    ]
  },
  {
    name: "asset.list",
    category: "Read",
    purpose: "List all indexed project assets registered in the asset manager with metadata.",
    schemaFile: "asset.list.json",
    paramsType: "AssetListParams",
    resultType: "AssetListResult",
    args: [
      { name: "asset_type", type: "string | null", required: false, description: "Optional type filter ('texture', 'audio', 'script', 'scene', 'tileset')", example: '"texture"' }
    ],
    resultDesc: "Array of AssetInfo records matching query filter.",
    resultExample: '{\n  "assets": [\n    {\n      "id": "asset-player-sprite",\n      "path": "assets/player.png",\n      "asset_type": "texture"\n    }\n  ]\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Asset database uninitialized", fix: "Verify assets directory exists" }
    ]
  },
  {
    name: "asset.get_info",
    category: "Read",
    purpose: "Retrieve detailed metadata and disk path for a specific asset UUID.",
    schemaFile: "asset.get_info.json",
    paramsType: "AssetGetInfoParams",
    resultType: "AssetGetInfoResult",
    args: [
      { name: "id", type: "string", required: true, description: "Stable UUID or asset ID string", example: '"asset-player-sprite"' }
    ],
    resultDesc: "AssetInfo record with id, file path, asset_type, and parsed metadata.",
    resultExample: '{\n  "asset": {\n    "id": "asset-player-sprite",\n    "path": "assets/player.png",\n    "asset_type": "texture",\n    "metadata": { "filter": "nearest" }\n  }\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Asset ID not found", fix: "Use asset.list to enumerate valid IDs" }
    ]
  },
  {
    name: "asset.import",
    category: "Write",
    purpose: "Import an external file into project assets and generate .meta sidecar descriptor.",
    schemaFile: "asset.import.json",
    paramsType: "AssetImportParams",
    resultType: "AssetImportResult",
    args: [
      { name: "source_path", type: "string", required: true, description: "Absolute path to source file", example: '"C:/Downloads/hero.png"' },
      { name: "target_path", type: "string", required: true, description: "Destination path relative to project assets", example: '"assets/hero.png"' },
      { name: "asset_type", type: "string | null", required: false, description: "Explicit asset type", example: '"texture"' }
    ],
    resultDesc: "Import result with assigned asset ID and saved path.",
    resultExample: '{\n  "success": true,\n  "asset_id": "asset-hero",\n  "path": "assets/hero.png"\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Source file unreadable or target path invalid", fix: "Verify source file exists and target directory is within project" }
    ]
  },
  {
    name: "physics.raycast",
    category: "Sim",
    purpose: "Cast a ray through Rapier 2D physics world and return the first collider intersection.",
    schemaFile: "physics.raycast.json",
    paramsType: "PhysicsRaycastParams",
    resultType: "PhysicsRaycastResult",
    args: [
      { name: "origin", type: "[number, number]", required: true, description: "Ray start position [x, y]", example: "[100.0, 50.0]" },
      { name: "direction", type: "[number, number]", required: true, description: "Ray direction unit vector [dx, dy]", example: "[0.0, -1.0]" },
      { name: "max_distance", type: "number | null", required: false, description: "Max raycast distance in world units (default 1000.0)", example: "200.0" },
      { name: "collision_mask", type: "number | null", required: false, description: "Layer collision bitmask", example: "1" },
      { name: "query_sensors", type: "boolean | null", required: false, description: "Whether to intersect sensor triggers", example: "false" }
    ],
    resultDesc: "Object containing hit details or null if no intersection occurred.",
    resultExample: '{\n  "hit": {\n    "entity_id": "e_ground_01",\n    "point": [100.0, 0.0],\n    "normal": [0.0, 1.0],\n    "fraction": 0.25\n  }\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Physics simulation world not stepped or zero direction vector", fix: "Ensure direction is non-zero vector" }
    ]
  },
  {
    name: "tilemap.get",
    category: "Read",
    purpose: "Query numeric tile ID at specific tilemap grid coordinate.",
    schemaFile: "tilemap.get.json",
    paramsType: "TilemapGetParams",
    resultType: "TilemapGetResult",
    args: [
      { name: "entity_id", type: "string", required: true, description: "Entity containing Tilemap component", example: '"e_map"' },
      { name: "layer", type: "string | null", required: false, description: "Layer name (defaults to 'main')", example: '"main"' },
      { name: "x", type: "number", required: true, description: "Integer grid X coordinate", example: "12" },
      { name: "y", type: "number", required: true, description: "Integer grid Y coordinate", example: "8" }
    ],
    resultDesc: "Object with tile ID (0 represents empty tile).",
    resultExample: '{\n  "tile_id": 4,\n  "layer": "main"\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Entity lacks Tilemap component", fix: "Verify entity has Tilemap component attached" }
    ]
  },
  {
    name: "tilemap.set_tile",
    category: "Write",
    purpose: "Set or clear tile ID at grid coordinates (x, y) on a tilemap layer.",
    schemaFile: "tilemap.set_tile.json",
    paramsType: "TilemapSetTileParams",
    resultType: "TilemapSetTileResult",
    args: [
      { name: "entity_id", type: "string", required: true, description: "Target entity ID", example: '"e_map"' },
      { name: "layer", type: "string | null", required: false, description: "Target layer name", example: '"main"' },
      { name: "x", type: "number", required: true, description: "Integer grid X coordinate", example: "12" },
      { name: "y", type: "number", required: true, description: "Integer grid Y coordinate", example: "8" },
      { name: "tile_id", type: "number", required: true, description: "Numeric tile ID (0 to erase)", example: "1" }
    ],
    resultDesc: "Object with old tile ID and confirmation.",
    resultExample: '{\n  "success": true,\n  "previous_tile_id": 0\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Entity has no Tilemap component", fix: "Add Tilemap component before modifying tiles" }
    ]
  },
  {
    name: "tilemap.fill_rect",
    category: "Write",
    purpose: "Fill rectangular region of tiles with given tile ID.",
    schemaFile: "tilemap.fill_rect.json",
    paramsType: "TilemapFillRectParams",
    resultType: "TilemapFillRectResult",
    args: [
      { name: "entity_id", type: "string", required: true, description: "Target entity ID", example: '"e_map"' },
      { name: "layer", type: "string | null", required: false, description: "Target layer", example: '"main"' },
      { name: "min", type: "[number, number]", required: true, description: "Minimum bounds [min_x, min_y]", example: "[0, 0]" },
      { name: "max", type: "[number, number]", required: true, description: "Maximum bounds [max_x, max_y]", example: "[10, 5]" },
      { name: "tile_id", type: "number", required: true, description: "Tile ID to fill", example: "2" }
    ],
    resultDesc: "Object reporting total number of modified tiles and chunks.",
    resultExample: '{\n  "success": true,\n  "tiles_modified": 66\n}',
    errors: [
      { code: -32602, meaning: "Invalid Params", commonCause: "min coordinates greater than max", fix: "Ensure min <= max in both X and Y dimensions" }
    ]
  },
  {
    name: "tilemap.clear",
    category: "Write",
    purpose: "Clear all tiles on all layers or on a specific layer of a tilemap.",
    schemaFile: "tilemap.clear.json",
    paramsType: "TilemapClearParams",
    resultType: "TilemapClearResult",
    args: [
      { name: "entity_id", type: "string", required: true, description: "Target entity ID", example: '"e_map"' },
      { name: "layer", type: "string | null", required: false, description: "Optional layer name (clears all layers if omitted)", example: '"decor"' }
    ],
    resultDesc: "Status confirming chunk clear count.",
    resultExample: '{\n  "success": true,\n  "chunks_cleared": 4\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Entity has no Tilemap component", fix: "Verify target entity" }
    ]
  },
  {
    name: "audio.play",
    category: "Exec",
    purpose: "Play sound effect or music asset through engine audio bus with volume/pitch/loop options.",
    schemaFile: "audio.play.json",
    paramsType: "AudioPlayParams",
    resultType: "AudioPlayResult",
    args: [
      { name: "asset_id", type: "string", required: true, description: "Audio asset ID or relative path", example: '"assets/jump.wav"' },
      { name: "bus", type: '"master" | "music" | "sfx"', required: false, description: "Target audio bus channel (default 'sfx')", example: '"sfx"' },
      { name: "volume", type: "number | null", required: false, description: "Volume scale (0.0 to 1.0, default 1.0)", example: "0.8" },
      { name: "pitch", type: "number | null", required: false, description: "Playback speed pitch factor (default 1.0)", example: "1.0" },
      { name: "loop", type: "boolean | null", required: false, description: "Whether to loop indefinitely", example: "false" }
    ],
    resultDesc: "Object containing unique playback instance ID.",
    resultExample: '{\n  "playback_id": "snd_inst_4821"\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Audio asset not found or unplayable format", fix: "Verify audio file format (WAV, OGG, MP3) and asset metadata" }
    ]
  },
  {
    name: "audio.stop",
    category: "Exec",
    purpose: "Stop an active sound instance or stop all audio playing on a specific bus.",
    schemaFile: "audio.stop.json",
    paramsType: "AudioStopParams",
    resultType: "AudioStopResult",
    args: [
      { name: "sound_id", type: "string | null", required: false, description: "Specific playback ID to stop", example: '"snd_inst_4821"' },
      { name: "bus", type: '"master" | "music" | "sfx" | null', required: false, description: "Stop all sounds routed to this bus", example: '"music"' }
    ],
    resultDesc: "Status confirming audio stop operation.",
    resultExample: '{\n  "stopped": true\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Invalid bus name specified", fix: "Use one of master, music, sfx" }
    ]
  },
  {
    name: "audio.set_bus_volume",
    category: "Write",
    purpose: "Set master, music, or sfx audio bus volume multiplier.",
    schemaFile: "audio.set_bus_volume.json",
    paramsType: "AudioSetBusVolumeParams",
    resultType: "AudioSetBusVolumeResult",
    args: [
      { name: "bus", type: '"master" | "music" | "sfx"', required: true, description: "Target bus channel", example: '"music"' },
      { name: "volume", type: "number", required: true, description: "Volume multiplier from 0.0 to 1.0", example: "0.5" }
    ],
    resultDesc: "Confirmation of updated bus volume.",
    resultExample: '{\n  "bus": "music",\n  "volume": 0.5\n}',
    errors: [
      { code: -32602, meaning: "Invalid Params", commonCause: "Volume outside 0.0 to 1.0 range", fix: "Clamp volume between 0.0 and 1.0" }
    ]
  },
  {
    name: "script.check",
    category: "Read",
    purpose: "Run TypeScript type checking on project scripts and return diagnostics.",
    schemaFile: "script.check.json",
    paramsType: "ScriptCheckParams",
    resultType: "ScriptCheckResult",
    args: [
      { name: "project_path", type: "string | null", required: false, description: "Project directory containing scripts/ (defaults to current dir)", example: '"templates/platformer"' }
    ],
    resultDesc: "Diagnostic report with errors, warnings, line numbers, and error codes.",
    resultExample: '{\n  "success": true,\n  "diagnostics": []\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "tsconfig.json missing or scripts directory unreadable", fix: "Ensure project contains scripts/ directory and tsconfig.json" }
    ]
  },
  {
    name: "script.list",
    category: "Read",
    purpose: "List all TypeScript source files located in the project's scripts folder.",
    schemaFile: "script.ops.json",
    paramsType: "ScriptListParams",
    resultType: "ScriptListResult",
    args: [
      { name: "project_path", type: "string | null", required: false, description: "Optional project root directory", example: '"."' }
    ],
    resultDesc: "List of relative script paths.",
    resultExample: '{\n  "scripts": ["scripts/player.ts", "scripts/camera.ts"]\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "scripts directory not found", fix: "Create scripts/ directory in project" }
    ]
  },
  {
    name: "script.read",
    category: "Read",
    purpose: "Read full text content of a TypeScript script file.",
    schemaFile: "script.ops.json",
    paramsType: "ScriptReadParams",
    resultType: "ScriptReadResult",
    args: [
      { name: "path", type: "string", required: true, description: "Project-relative path to script", example: '"scripts/player.ts"' }
    ],
    resultDesc: "Object containing script text content.",
    resultExample: '{\n  "content": "export function onStart() { forge.log.info(\\"Ready\\"); }"\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Script file not found", fix: "Verify script path using script.list" }
    ]
  },
  {
    name: "script.write",
    category: "Write",
    purpose: "Validate TypeScript syntax using SWC parser and write content to script file.",
    schemaFile: "script.ops.json",
    paramsType: "ScriptWriteParams",
    resultType: "ScriptWriteResult",
    args: [
      { name: "path", type: "string", required: true, description: "Target script path", example: '"scripts/player.ts"' },
      { name: "content", type: "string", required: true, description: "TypeScript script source code", example: '"export function onStart() {}"' }
    ],
    resultDesc: "Confirmation of write operation.",
    resultExample: '{\n  "success": true\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Syntax error in TypeScript content", fix: "Fix TypeScript syntax error before saving" }
    ]
  },
  {
    name: "play.start",
    category: "Sim",
    purpose: "Snapshot active scene and begin real-time play mode simulation (physics, scripts, timers).",
    schemaFile: "play.ops.json",
    paramsType: "PlayStartParams",
    resultType: "PlayStartResult",
    args: [],
    resultDesc: "Play state confirmation ('playing').",
    resultExample: '{\n  "success": true,\n  "state": "playing"\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Already in play mode", fix: "Call play.stop first before restarting" }
    ]
  },
  {
    name: "play.pause",
    category: "Sim",
    purpose: "Pause active simulation, freezing physics and scripts while keeping viewport renderable.",
    schemaFile: "play.ops.json",
    paramsType: "PlayPauseParams",
    resultType: "PlayPauseResult",
    args: [],
    resultDesc: "Play state confirmation ('paused').",
    resultExample: '{\n  "success": true,\n  "state": "paused"\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Engine not currently in play mode", fix: "Start play mode first with play.start" }
    ]
  },
  {
    name: "play.step",
    category: "Sim",
    purpose: "Advance simulation by exactly N frames while paused, then pause again.",
    schemaFile: "play.ops.json",
    paramsType: "PlayStepParams",
    resultType: "PlayStepResult",
    args: [
      { name: "frames", type: "number", required: false, description: "Number of discrete simulation frames to advance (default 1)", example: "1" }
    ],
    resultDesc: "Confirmation with total frames stepped.",
    resultExample: '{\n  "success": true,\n  "frames_stepped": 1\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Simulation is not paused", fix: "Pause simulation before stepping" }
    ]
  },
  {
    name: "play.stop",
    category: "Sim",
    purpose: "Stop play mode and restore exact scene state snapshot captured before play started.",
    schemaFile: "play.ops.json",
    paramsType: "PlayStopParams",
    resultType: "PlayStopResult",
    args: [],
    resultDesc: "Confirmation of restoration to edit mode ('stopped').",
    resultExample: '{\n  "success": true,\n  "state": "stopped"\n}',
    errors: [
      { code: -32000, meaning: "Engine Error", commonCause: "Not in play mode", fix: "No action required" }
    ]
  }
];

export function generateBridgeTools(): string[] {
  const generatedIds: string[] = [];

  for (const tool of BRIDGE_TOOLS) {
    const id = `ref-tool-${tool.name.replace(/[^a-z0-9]/gi, "-")}`;
    generatedIds.push(id);

    const argsTable = tool.args.length > 0
      ? `| Argument | Type | Required | Description | Example |\n| :--- | :--- | :--- | :--- | :--- |\n` +
        tool.args.map(a => `| \`${a.name}\` | \`${a.type}\` | ${a.required ? "**Yes**" : "No"} | ${a.description} | \`${a.example || "-"}\` |`).join("\n")
      : "_This tool takes no arguments._";

    const errorsTable = tool.errors.length > 0
      ? `| Code | Meaning | Common Cause | Recommended Fix |\n| :--- | :--- | :--- | :--- |\n` +
        tool.errors.map(e => `| \`${e.code}\` | **${e.meaning}** | ${e.commonCause} | ${e.fix} |`).join("\n")
      : "_No tool-specific error codes reported._";

    const md = `---
id: ${id}
title: "Bridge Tool: ${tool.name}"
summary: "${tool.purpose}"
tags: ["reference", "bridge-tool", "${tool.category.toLowerCase()}"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["ref-tools-index", "concept-architecture", "agents-overview"]
---

# \`${tool.name}\`

**Category:** \`${tool.category}\`  
**Purpose:** ${tool.purpose}

---

## Arguments

${argsTable}

---

## Result

**Return Type:** \`${tool.resultType || "object"}\`  
${tool.resultDesc}

\`\`\`json
${tool.resultExample}
\`\`\`

---

## Error Catalog

${errorsTable}

---

## Usage Example

\`\`\`snippet
// Client invocation of ${tool.name}
console.log("Tool: ${tool.name}");
\`\`\`
`;

    const outPath = path.join(REF_DIR, "bridge-tools", `${tool.name}.md`);
    fs.writeFileSync(outPath, md, "utf-8");
  }

  // Generate bridge tools index page
  const indexId = "ref-tools-index";
  generatedIds.push(indexId);
  const indexRows = BRIDGE_TOOLS.map(t =>
    `| [\`${t.name}\`](file:///docs/src/reference/bridge-tools/${t.name}.md) | \`${t.category}\` | ${t.purpose} |`
  ).join("\n");

  const indexMd = `---
id: ${indexId}
title: "Bridge Tools Reference Index"
summary: "Complete reference catalogue for all Forge JSON-RPC 2.0 and MCP bridge tools."
tags: ["reference", "tools", "index"]
audience: both
engine_version: "0.1.0"
last_verified: "2026-10-27"
related_ids: ["ref-components-index", "ref-events-index", "ref-errors-index"]
---

# Bridge Tools Reference Index

Forge exposes a bidirectional JSON-RPC 2.0 and MCP interface on port 9002 for IDE integration and autonomous agent control.

| Tool Name | Category | Purpose |
| :--- | :--- | :--- |
${indexRows}
`;
  fs.writeFileSync(path.join(REF_DIR, "bridge-tools", "index.md"), indexMd, "utf-8");

  return generatedIds;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  generateBridgeTools();
  console.log(`Generated ${BRIDGE_TOOLS.length} bridge tool references.`);
}
