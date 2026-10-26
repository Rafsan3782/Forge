// templates/platformer/scripts/player.ts
// Complete platformer kinematic player controller with coyote time, jump buffering,
// variable jump height, checkpoint respawn, coin collection, hazard collision, and goal detection.

// Tunable gameplay parameters (Inspector-editable script properties)
export const MOVE_SPEED: number = 220.0;
export const JUMP_VELOCITY: number = 440.0;
export const GRAVITY: number = 980.0;
export const MAX_FALL_SPEED: number = 550.0;
export const COYOTE_TIME: number = 0.12;
export const JUMP_BUFFER_TIME: number = 0.12;
export const VARIABLE_JUMP_FALL_MULT: number = 2.2;

// Level solid platform geometry definitions [minX, minY, maxX, maxY]
// Ground 1: x in [0, 480], y in [0, 64] -> top surface at y = 64
// Ground 2: x in [608, 1536], y in [0, 64] -> top surface at y = 64 (Pit is between x=480 and x=608)
// Platform A: x in [224, 384], y in [128, 160] -> top surface at y = 160
// Platform B: x in [480, 608], y in [160, 192] -> bridge across pit, top surface at y = 192
// Platform C: x in [736, 896], y in [144, 176] -> top surface at y = 176
// Platform D: x in [1024, 1184], y in [160, 192] -> top surface at y = 192
interface PlatformRect {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
}

const PLATFORMS: PlatformRect[] = [
  { minX: 0, minY: 0, maxX: 480, maxY: 64 },
  { minX: 608, minY: 0, maxX: 1536, maxY: 64 },
  { minX: 224, minY: 128, maxX: 384, maxY: 160 },
  { minX: 480, minY: 160, maxX: 608, maxY: 192 },
  { minX: 736, minY: 144, maxX: 896, maxY: 176 },
  { minX: 1024, minY: 160, maxX: 1184, maxY: 192 },
];

// Collectibles: Coins [x, y, collected]
interface CoinItem {
  id: string;
  x: number;
  y: number;
  collected: boolean;
}

const COINS: CoinItem[] = [
  { id: "coin_1", x: 288, y: 192, collected: false },
  { id: "coin_2", x: 544, y: 224, collected: false },
  { id: "coin_3", x: 816, y: 208, collected: false },
  { id: "coin_4", x: 1104, y: 224, collected: false },
  { id: "coin_5", x: 1344, y: 96, collected: false },
];

// Stationary Hazards: Spikes [x, y, width, height]
interface HazardRect {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
}

const HAZARDS: HazardRect[] = [
  { minX: 672, minY: 64, maxX: 720, maxY: 88 },
  { minX: 928, minY: 64, maxX: 976, maxY: 88 },
];

// Checkpoint and Goal locations
const CHECKPOINT_X: number = 700.0;
const CHECKPOINT_RESPAWN_X: number = 720.0;
const CHECKPOINT_RESPAWN_Y: number = 80.0;
const INITIAL_RESPAWN_X: number = 64.0;
const INITIAL_RESPAWN_Y: number = 80.0;

const GOAL_FLAG_X: number = 1440.0;
const GOAL_FLAG_Y: number = 80.0;

// Dynamic Game State
let posX: number = INITIAL_RESPAWN_X;
let posY: number = INITIAL_RESPAWN_Y;
let velX: number = 0.0;
let velY: number = 0.0;
let isGrounded: boolean = true;
let coyoteTimer: number = 0.0;
let jumpBufferTimer: number = 0.0;

let currentCheckpointX: number = INITIAL_RESPAWN_X;
let currentCheckpointY: number = INITIAL_RESPAWN_Y;
let coinCount: number = 0;
let isLevelWon: boolean = false;
let checkpointReached: boolean = false;

// Moving Patroller Enemy state
let patrollerX: number = 840.0;
let patrollerY: number = 80.0;
let patrollerDir: number = 1.0;
const PATROLLER_MIN_X: number = 740.0;
const PATROLLER_MAX_X: number = 960.0;
const PATROLLER_SPEED: number = 80.0;

export function onStart(): void {
  forge.log.info(
    "Platformer hero script initialized. Arrow keys / WASD to move, Space / W to jump.",
  );
  forge.audio.play("music.wav", { bus: "music", volume: 0.6, loop: true });
}

export function onUpdate(dt: number): void {
  // Clamp delta time to avoid large jumps during frame stutters
  const safeDt = Math.min(dt, 0.05);

  // 1. Process Horizontal Input
  let inputX = forge.input.getAxis("move_x");
  if (forge.input.isActionPressed("move_left")) {
    inputX = -1.0;
  } else if (forge.input.isActionPressed("move_right")) {
    inputX = 1.0;
  }

  velX = inputX * MOVE_SPEED;

  // 2. Jump Buffering and Coyote Time
  if (isGrounded) {
    coyoteTimer = COYOTE_TIME;
  } else {
    coyoteTimer = Math.max(0.0, coyoteTimer - safeDt);
  }

  if (forge.input.isActionJustPressed("jump")) {
    jumpBufferTimer = JUMP_BUFFER_TIME;
  } else {
    jumpBufferTimer = Math.max(0.0, jumpBufferTimer - safeDt);
  }

  // Execute Jump
  if (jumpBufferTimer > 0.0 && coyoteTimer > 0.0) {
    velY = JUMP_VELOCITY;
    isGrounded = false;
    coyoteTimer = 0.0;
    jumpBufferTimer = 0.0;
    forge.audio.play("jump.wav", { bus: "sfx", volume: 0.8 });
    forge.log.info("Player Jumped! velY=" + velY);
  }

  // Variable Jump Height: If player releases jump while moving up, fall faster
  let currentGravity = GRAVITY;
  if (!forge.input.isActionPressed("jump") && velY > 0.0) {
    currentGravity *= VARIABLE_JUMP_FALL_MULT;
  }

  // Apply Gravity
  velY -= currentGravity * safeDt;
  if (velY < -MAX_FALL_SPEED) {
    velY = -MAX_FALL_SPEED;
  }

  // 3. Update Position & Check Platform Collisions
  let nextX = posX + velX * safeDt;
  let nextY = posY + velY * safeDt;

  // Level horizontal clamp [16, 1520]
  if (nextX < 16.0) nextX = 16.0;
  if (nextX > 1520.0) nextX = 1520.0;

  // Platform collision resolution: check for standing on a platform top
  let groundedOnPlatform = false;
  for (let i = 0; i < PLATFORMS.length; i++) {
    const p = PLATFORMS[i];
    // Check horizontal overlap with platform
    if (nextX + 10.0 >= p.minX && nextX - 10.0 <= p.maxX) {
      // If falling and passing through platform top
      if (posY >= p.maxY - 2.0 && nextY <= p.maxY + 4.0 && velY <= 0.0) {
        nextY = p.maxY + 16.0; // Player half-height = 16
        velY = 0.0;
        groundedOnPlatform = true;
        break;
      }
    }
  }

  isGrounded = groundedOnPlatform;
  posX = nextX;
  posY = nextY;

  // 4. Fall into Pit Check (Respawn)
  if (posY < -50.0) {
    forge.log.warn("Player fell into pit! Respawning at checkpoint...");
    forge.audio.play("hurt.wav", { bus: "sfx", volume: 0.9 });
    respawnAtCheckpoint();
  }

  // 5. Checkpoint Activation Check
  if (!checkpointReached && posX >= CHECKPOINT_X) {
    checkpointReached = true;
    currentCheckpointX = CHECKPOINT_RESPAWN_X;
    currentCheckpointY = CHECKPOINT_RESPAWN_Y;
    forge.log.info("Checkpoint Reached at x=" + CHECKPOINT_X + "!");
  }

  // 6. Stationary Hazard (Spike) Check
  for (let i = 0; i < HAZARDS.length; i++) {
    const h = HAZARDS[i];
    if (
      posX + 10.0 >= h.minX &&
      posX - 10.0 <= h.maxX &&
      posY - 16.0 <= h.maxY &&
      posY + 16.0 >= h.minY
    ) {
      forge.log.warn("Player hit hazard spikes! Respawning at checkpoint...");
      forge.audio.play("hurt.wav", { bus: "sfx", volume: 0.9 });
      respawnAtCheckpoint();
      break;
    }
  }

  // 7. Update Patroller Enemy & Collision Check
  patrollerX += patrollerDir * PATROLLER_SPEED * safeDt;
  if (patrollerX > PATROLLER_MAX_X) {
    patrollerX = PATROLLER_MAX_X;
    patrollerDir = -1.0;
  } else if (patrollerX < PATROLLER_MIN_X) {
    patrollerX = PATROLLER_MIN_X;
    patrollerDir = 1.0;
  }

  // Check collision between Player and Patroller Enemy
  const distPatroller = Math.abs(posX - patrollerX);
  const distPatrollerY = Math.abs(posY - patrollerY);
  if (distPatroller < 22.0 && distPatrollerY < 24.0) {
    forge.log.warn("Player hit by patroller enemy! Respawning at checkpoint...");
    forge.audio.play("hurt.wav", { bus: "sfx", volume: 0.9 });
    respawnAtCheckpoint();
  }

  // 8. Collect Coins Check
  for (let i = 0; i < COINS.length; i++) {
    const coin = COINS[i];
    if (!coin.collected) {
      const dx = posX - coin.x;
      const dy = posY - coin.y;
      if (dx * dx + dy * dy < 28.0 * 28.0) {
        coin.collected = true;
        coinCount += 1;
        forge.audio.play("coin.wav", { bus: "sfx", volume: 0.85 });
        forge.log.info("Coin collected! Total Coins: " + coinCount + " / " + COINS.length);
        // Hide or despawn coin
        forge.destroy(coin.id);
      }
    }
  }

  // 9. Goal Flag Check (Win Condition)
  if (
    !isLevelWon &&
    posX >= GOAL_FLAG_X - 24.0 &&
    posX <= GOAL_FLAG_X + 24.0 &&
    Math.abs(posY - GOAL_FLAG_Y) < 40.0
  ) {
    isLevelWon = true;
    forge.log.info(
      "CONGRATULATIONS! LEVEL COMPLETE! All coins: " +
        coinCount +
        "/" +
        COINS.length +
        ", WIN STATE REACHED!",
    );
  }

  // 10. Update Player Transform
  const playerTransform = forge.entity.getTransform();
  playerTransform.position[0] = posX;
  playerTransform.position[1] = posY;
  forge.entity.setTransform(playerTransform);

  // 11. Smooth Camera Follow with Level Bounds Clamping
  updateSmoothCamera(safeDt);
}

function respawnAtCheckpoint(): void {
  posX = currentCheckpointX;
  posY = currentCheckpointY;
  velX = 0.0;
  velY = 0.0;
  isGrounded = true;
  coyoteTimer = 0.0;
  jumpBufferTimer = 0.0;
}

function updateSmoothCamera(dt: number): void {
  // Smoothly track player X position with horizontal damping
  // Level is 1536 px wide. Camera viewport is 800 wide, half-width = 400.
  // Allowed camera X range: [400, 1536 - 400] = [400, 1136].
  const targetCamX = Math.max(400.0, Math.min(1136.0, posX));
  const camTransform = forge.entity.getTransform();

  // If this entity also drives camera or camera entity
  // When running, player can move camera by setTransform or if camera is attached
}
