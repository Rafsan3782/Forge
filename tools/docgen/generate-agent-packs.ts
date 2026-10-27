import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, "../..");
const DIST_AGENT_PACKS = path.join(REPO_ROOT, "docs/dist/agent-packs");

fs.mkdirSync(DIST_AGENT_PACKS, { recursive: true });

export interface AgentPackTopic {
  name: string;
  title: string;
  description: string;
  coreConcepts: string[];
  keyTools: string[];
  keyComponents: string[];
  invariants: string[];
  commonMistakes: string[];
  runnableExample: string;
}

export const AGENT_PACK_TOPICS: AgentPackTopic[] = [
  {
    name: "scripting-basics",
    title: "Scripting Basics & Lifecycle",
    description: "Compact brief on authoring TypeScript scripts, lifecycle callbacks, and global forge API access.",
    coreConcepts: [
      "Scripts execute in embedded QuickJS on the engine thread.",
      "Attached to entities via the Script component with a relative path.",
      "State can be preserved across hot reloads via serialize() and deserialize()."
    ],
    keyTools: ["script.check", "script.list", "script.read", "script.write"],
    keyComponents: ["Script", "Transform2D", "Name"],
    invariants: [
      "onUpdate(dt) dt is frame delta in seconds, always positive.",
      "forge.entity methods always query the entity executing the script.",
      "SWC syntax check must pass before script write succeeds."
    ],
    commonMistakes: [
      "Calling window or DOM APIs: QuickJS does not provide browser globals.",
      "Forgetting to export lifecycle hooks: export function onUpdate(dt: number) must be exported.",
      "Mutating transform.position without calling forge.entity.setTransform(transform)."
    ],
    runnableExample: `export function onUpdate(dt: number): void {
  const transform = forge.entity.getTransform();
  if (forge.input.isActionPressed("move_right")) {
    transform.position[0] += 120 * dt;
    forge.entity.setTransform(transform);
  }
}`
  },
  {
    name: "scene-editing",
    title: "Scene Editing & Hierarchy Manipulation",
    description: "Brief on programmatic entity creation, hierarchy reparenting, and component modification.",
    coreConcepts: [
      "Scenes are serialized as JSON hierarchies of entities with attached components.",
      "Entity IDs are stable UUIDs; parent references model parent-child trees.",
      "Mutations can be performed transactionally via bridge tools or scene.edit.ops."
    ],
    keyTools: ["scene.get_tree", "scene.create_entity", "scene.set_component", "scene.delete_entity"],
    keyComponents: ["Name", "Transform2D", "Sprite", "Camera2D"],
    invariants: [
      "Entity deletion recursively removes all child descendants.",
      "Every entity has at least Name and Transform2D components.",
      "Modifying an entity marks the scene dirty in the undo stack."
    ],
    commonMistakes: [
      "Using non-existent component names: only registered components can be set.",
      "Passing vector2 as an object instead of [x, y] tuple.",
      "Hardcoding temporary entity IDs across session restarts."
    ],
    runnableExample: `// Programmatic entity spawning via script API
const coinId = forge.spawn("Coin", {
  "Transform2D": { "position": [150.0, 80.0] },
  "Sprite": { "asset_id": "assets/coin.png", "size": [16.0, 16.0] }
});`
  },
  {
    name: "physics",
    title: "2D Physics & Collision Simulation",
    description: "Brief on Rapier 2D rigid bodies, collider shapes, friction, restitution, and raycast queries.",
    coreConcepts: [
      "Physics is stepped at 60Hz constant delta time.",
      "RigidBody2D determines simulation dynamics (dynamic, fixed, kinematic).",
      "Collider2D defines geometry (box, circle) and sensor trigger behavior."
    ],
    keyTools: ["physics.raycast", "play.start", "play.pause", "play.step"],
    keyComponents: ["RigidBody2D", "Collider2D", "Transform2D"],
    invariants: [
      "Sensors detect overlaps without exerting contact impulse forces.",
      "Raycast direction must be a non-zero vector; hit fractions range from 0.0 to 1.0.",
      "lock_rotation must be set to true on upright characters."
    ],
    commonMistakes: [
      "Attaching Collider2D without RigidBody2D on dynamic bodies.",
      "Setting zero or negative half-extents on collider sizes.",
      "Querying physics while simulation is stopped."
    ],
    runnableExample: `// Raycast ground check
const hit = forge.physics.raycast([100, 50], [0, -1], 10.0);
if (hit) {
  forge.log.info("Grounded on entity: " + hit.entity_id);
}`
  },
  {
    name: "tilemaps",
    title: "Tilemap Systems & Chunked Grids",
    description: "Brief on grid-aligned tile graphics, chunks, layer queries, and procedural level editing.",
    coreConcepts: [
      "Tilemaps store tile indices in sparse spatial chunks for fast rendering.",
      "Tile ID 0 denotes an empty tile; positive integers index tileset atlas frames.",
      "Tilemaps support multiple named layers (e.g. background, main, foreground)."
    ],
    keyTools: ["tilemap.get", "tilemap.set_tile", "tilemap.fill_rect", "tilemap.clear"],
    keyComponents: ["Tilemap", "Transform2D"],
    invariants: [
      "Chunk coordinates are integer multiples of chunk_size (default 16).",
      "Layer names are case-sensitive; defaults to 'main'.",
      "fill_rect min bounds must be strictly <= max bounds."
    ],
    commonMistakes: [
      "Passing floating point numbers as tile coordinates instead of integers.",
      "Calling tilemap tools on entities without a Tilemap component.",
      "Forgetting to specify tileset_id asset path."
    ],
    runnableExample: `// Procedural platform placement
for (let x = 0; x < 20; x++) {
  forge.tilemap.setTile(mapId, "main", x, 0, 1);
}`
  },
  {
    name: "animation",
    title: "Spritesheet Animation & State Management",
    description: "Brief on keyframe clip playback, frame durations, looping, and state transitions.",
    coreConcepts: [
      "Clips slice a shared spritesheet texture into indexed frame rectangles.",
      "Playback speed and loop state are controlled via script or Animator component.",
      "Clip transitions preserve current entity transform."
    ],
    keyTools: ["scene.set_component", "play.start"],
    keyComponents: ["Sprite", "Animator", "Transform2D"],
    invariants: [
      "Active clip names must match definitions inside the spritesheet .meta file.",
      "Frame durations are positive values in seconds."
    ],
    commonMistakes: [
      "Calling animation.play with a clip name that does not exist.",
      "Triggering play every frame inside onUpdate(dt) without checking current clip."
    ],
    runnableExample: `if (forge.input.isActionJustPressed("move_right")) {
  forge.animation.play("run");
}`
  },
  {
    name: "audio",
    title: "Audio Playback & Bus Mixing",
    description: "Brief on playing sound effects and music tracks across master, music, and sfx channels.",
    coreConcepts: [
      "Audio is routed to one of three buses: master, music, or sfx.",
      "play returns a unique playback instance ID that can be stopped individually.",
      "Bus volume controls attenuate all active tracks on that channel."
    ],
    keyTools: ["audio.play", "audio.stop", "audio.set_bus_volume"],
    keyComponents: ["AudioSource", "AudioListener"],
    invariants: [
      "Volume values are clamped between 0.0 (silent) and 1.0 (full).",
      "Audio files must be valid WAV, OGG, or MP3 assets."
    ],
    commonMistakes: [
      "Spawning looping audio without retaining the playback ID to stop it later.",
      "Playing sound every frame inside onUpdate(dt) instead of on action triggers."
    ],
    runnableExample: `const sfxId = forge.audio.play("assets/jump.wav", {
  bus: "sfx",
  volume: 0.8,
  loop: false
});`
  },
  {
    name: "input",
    title: "Input Mapping & Polling",
    description: "Brief on action bindings, analog axes, and frame-accurate state polling.",
    coreConcepts: [
      "Input mappings are defined in input_map.json linking actions to hardware keys.",
      "isActionJustPressed returns true on key down frame only.",
      "isActionPressed returns true as long as key is held.",
      "getAxis returns digital or analog range -1.0 to 1.0."
    ],
    keyTools: ["game.simulate_input"],
    keyComponents: ["Script"],
    invariants: [
      "Action names match input_map.json exactly (e.g. 'move_left', 'jump').",
      "Input is updated at the beginning of each simulation frame before onUpdate."
    ],
    commonMistakes: [
      "Using isActionJustPressed for continuous movement instead of isActionPressed.",
      "Calling isActionPressed with raw hardware key codes instead of logical action names."
    ],
    runnableExample: `const moveX = forge.input.getAxis("horizontal");
const isJumping = forge.input.isActionJustPressed("jump");`
  },
  {
    name: "debugging",
    title: "Headless Testing, Telemetry & Diagnostics",
    description: "Brief on headless frame capture, automated playtest reports, and script diagnostics.",
    coreConcepts: [
      "Engine can run completely offscreen via --headless mode.",
      "game.screenshot captures viewport buffer to file or base64.",
      "forge.log outputs structured events streamed to IDE and agents.",
      "forge check validates TypeScript scripts with line numbers."
    ],
    keyTools: ["script.check", "game.screenshot", "game.simulate_input", "engine.ping"],
    keyComponents: ["Script"],
    invariants: [
      "Headless mode produces identical simulation state to interactive mode.",
      "Script errors emit game.error notifications containing mapped line numbers."
    ],
    commonMistakes: [
      "Attempting to capture screenshot before scene has rendered initial frame.",
      "Ignoring TypeScript warnings from forge check before starting play mode."
    ],
    runnableExample: `forge.log.info("Telemetry state:", {
  fps: 60,
  entities: 24,
  playerPos: forge.entity.getTransform().position
});`
  },
  {
    name: "game-feel",
    title: "Game Feel, Juice & Polish",
    description: "Brief on camera smoothing, jump buffering, coyote time, and visual feedback.",
    coreConcepts: [
      "Coyote time allows jumping briefly after walking off ledges (~0.1s).",
      "Jump buffering records jump inputs right before landing (~0.15s).",
      "Camera lerp smooths target tracking to prevent jarring camera cuts."
    ],
    keyTools: ["scene.set_component", "audio.play"],
    keyComponents: ["Camera2D", "Transform2D", "Sprite"],
    invariants: [
      "Coyote time counter decrements by dt each frame when ungrounded.",
      "Camera zoom must remain positive (> 0.1)."
    ],
    commonMistakes: [
      "Instant 1:1 camera tracking that induces motion fatigue.",
      "Strict single-frame jump timing that feels unresponsive to human players."
    ],
    runnableExample: `// Smooth camera tracking
const cam = forge.entity.getTransform();
cam.position[0] += (targetX - cam.position[0]) * 5.0 * dt;
forge.entity.setTransform(cam);`
  }
];

export function generateAgentPacks(): Array<{ name: string; path: string; tokenCount: number }> {
  const results: Array<{ name: string; path: string; tokenCount: number }> = [];

  for (const topic of AGENT_PACK_TOPICS) {
    const content = `# Agent Pack: ${topic.title}

> Topic: \`${topic.name}\`  
> Target: Autonomous AI Agents & Fast Retrieval  
> Scope: Invariants, tool signatures, components, and minimal examples without filler.

---

## 1. Core Architectural Concepts

${topic.coreConcepts.map(c => `- ${c}`).join("\n")}

---

## 2. Key Bridge Tools

${topic.keyTools.map(t => `- \`${t}\``).join("\n")}

---

## 3. Related ECS Components

${topic.keyComponents.map(c => `- \`${c}\``).join("\n")}

---

## 4. Invariants & Rules

${topic.invariants.map(inv => `- **Invariant:** ${inv}`).join("\n")}

---

## 5. Common Mistakes to Avoid

${topic.commonMistakes.map(m => `- **Mistake:** ${m}`).join("\n")}

---

## 6. Minimal Runnable Example

\`\`\`ts
${topic.runnableExample}
\`\`\`
`;

    const tokenCount = estimateTokens(content);
    if (tokenCount > 3000) {
      console.warn(`WARNING: Agent pack '${topic.name}' exceeds 3000 tokens (${tokenCount} tokens).`);
    }

    const outPath = path.join(DIST_AGENT_PACKS, `${topic.name}.md`);
    fs.writeFileSync(outPath, content, "utf-8");

    results.push({
      name: topic.name,
      path: outPath,
      tokenCount
    });
  }

  return results;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const packs = generateAgentPacks();
  console.log(`Generated ${packs.length} agent packs.`);
  for (const p of packs) {
    console.log(` - ${p.name}: ${p.tokenCount} tokens`);
  }
}
