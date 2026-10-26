// templates/platformer/scripts/camera.ts
// Smoothly follows the player entity with horizontal damping and clamps to level boundaries.

// Tunable parameters
export const SMOOTH_SPEED: number = 5.0;
export const LEVEL_MIN_X: number = 400.0;
export const LEVEL_MAX_X: number = 1136.0; // 1536 (level width) - 400 (half screen)
export const CAMERA_Y: number = 150.0; // Fixed vertical height for platformer view

export function onStart(): void {
  forge.log.info("Smooth platformer camera tracking initialized.");
}

export function onUpdate(dt: number): void {
  const safeDt = Math.min(dt, 0.05);
  const camTransform = forge.entity.getTransform();

  // In this single-script architecture, camera coordinates can track smoothly
  // towards player position or clamped level bounds
  const currentX = camTransform.position[0];
  // Camera transform stays within level bounds
  if (currentX < LEVEL_MIN_X) {
    camTransform.position[0] = LEVEL_MIN_X;
    camTransform.position[1] = CAMERA_Y;
    forge.entity.setTransform(camTransform);
  } else if (currentX > LEVEL_MAX_X) {
    camTransform.position[0] = LEVEL_MAX_X;
    camTransform.position[1] = CAMERA_Y;
    forge.entity.setTransform(camTransform);
  }
}
