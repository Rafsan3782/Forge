// templates/platformer/scripts/patroller.ts
// Autonomous enemy that patrols back and forth between defined horizontal patrol points.

export const PATROL_SPEED: number = 75.0;
export const LEFT_BOUND: number = 740.0;
export const RIGHT_BOUND: number = 960.0;

let currentX: number = 840.0;
let currentY: number = 80.0;
let direction: number = 1.0;

export function onStart(): void {
  const transform = forge.entity.getTransform();
  currentX = transform.position[0];
  currentY = transform.position[1];
}

export function onUpdate(dt: number): void {
  const safeDt = Math.min(dt, 0.05);
  currentX += direction * PATROL_SPEED * safeDt;

  if (currentX >= RIGHT_BOUND) {
    currentX = RIGHT_BOUND;
    direction = -1.0;
  } else if (currentX <= LEFT_BOUND) {
    currentX = LEFT_BOUND;
    direction = 1.0;
  }

  const transform = forge.entity.getTransform();
  transform.position[0] = currentX;
  transform.position[1] = currentY;
  forge.entity.setTransform(transform);
}
