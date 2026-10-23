// templates/script-demo/scripts/player.ts
// Player hero controller script using forge API

const SPEED: number = 150.0;

export function onStart(): void {
  forge.log.info("Player script started for entity: " + forge.entity.getName());
}

export function onUpdate(dt: number): void {
  const transform = forge.entity.getTransform();
  let dx: number = 0.0;
  let dy: number = 0.0;

  if (forge.input.isActionPressed("move_left")) {
    dx -= SPEED * dt;
  }
  if (forge.input.isActionPressed("move_right")) {
    dx += SPEED * dt;
  }
  if (forge.input.isActionPressed("move_up")) {
    dy += SPEED * dt;
  }
  if (forge.input.isActionPressed("move_down")) {
    dy -= SPEED * dt;
  }

  if (dx !== 0.0 || dy !== 0.0) {
    transform.position[0] += dx;
    transform.position[1] += dy;
    forge.entity.setTransform(transform);
  }
}
