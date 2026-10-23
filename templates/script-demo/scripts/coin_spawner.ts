// templates/script-demo/scripts/coin_spawner.ts
// Spawns coins on a timer using forge.spawn

let timer: number = 0.0;
const SPAWN_INTERVAL: number = 1.0;
let coinCount: number = 0;

export function onStart(): void {
  forge.log.info("Coin spawner initialized.");
}

export function onUpdate(dt: number): void {
  timer += dt;
  if (timer >= SPAWN_INTERVAL) {
    timer -= SPAWN_INTERVAL;
    coinCount += 1;
    const coinId = forge.spawn("Coin_" + coinCount);
    forge.log.info("Spawned new coin: " + coinId);
  }
}
