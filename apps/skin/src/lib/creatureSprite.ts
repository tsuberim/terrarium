import type { Creature } from "./api";

export type SpriteMode = "id" | "hash";

const SIZE = 8;

export type CreatureSprite = {
  /** row-major 8×8 body pixels */
  body: boolean[];
  bodyColor: string;
  accentColor: string;
  eyeColor: string;
};

const cache = new Map<string, CreatureSprite>();

function fnv1a(str: string): number {
  let h = 2166136261;
  for (let i = 0; i < str.length; i++) {
    h ^= str.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

function byte(seed: string, i: number): number {
  return fnv1a(`${seed}:${i}`) & 0xff;
}

function hsl(h: number, s: number, l: number): string {
  return `hsl(${h % 360} ${s}% ${l}%)`;
}

/** ID = every creature unique; hash = same program shares appearance. */
export function spriteSeed(creature: Creature, mode: SpriteMode): string {
  if (mode === "hash" && creature.program_hash) {
    return creature.program_hash;
  }
  return creature.id;
}

export function creatureSprite(creature: Creature, mode: SpriteMode): CreatureSprite {
  const seed = spriteSeed(creature, mode);
  const cached = cache.get(seed);
  if (cached) return cached;

  const body: boolean[] = new Array(SIZE * SIZE).fill(false);
  for (let y = 0; y < SIZE; y++) {
    for (let x = 0; x < 4; x++) {
      const bit = (byte(seed, y) >> x) & 1;
      if (!bit) continue;
      body[y * SIZE + x] = true;
      body[y * SIZE + (SIZE - 1 - x)] = true;
    }
  }

  // Simple belly + eyes so blobs read as critters.
  for (let y = 4; y < 7; y++) {
    for (let x = 2; x < 6; x++) {
      body[y * SIZE + x] = true;
    }
  }
  body[2 * SIZE + 2] = false;
  body[2 * SIZE + 5] = false;
  body[3 * SIZE + 2] = true;
  body[3 * SIZE + 5] = true;

  const hue = byte(seed, 99) % 360;
  const sprite: CreatureSprite = {
    body,
    bodyColor: hsl(hue, 58, 48),
    accentColor: hsl(hue + 40, 70, 62),
    eyeColor: hsl(hue + 180, 20, 92),
  };
  cache.set(seed, sprite);
  return sprite;
}

export function drawCreatureSprite(
  ctx: CanvasRenderingContext2D,
  cellX: number,
  cellY: number,
  cellPx: number,
  sprite: CreatureSprite,
  mine: boolean,
) {
  const px = 2;
  const ox = cellX * cellPx + Math.floor((cellPx - SIZE * px) / 2);
  const oy = cellY * cellPx + Math.floor((cellPx - SIZE * px) / 2);

  ctx.fillStyle = mine ? "rgba(74, 232, 194, 0.12)" : "rgba(123, 109, 255, 0.12)";
  ctx.fillRect(cellX * cellPx + 1, cellY * cellPx + 1, cellPx - 2, cellPx - 2);

  for (let y = 0; y < SIZE; y++) {
    for (let x = 0; x < SIZE; x++) {
      if (!sprite.body[y * SIZE + x]) continue;
      const isEye = (x === 2 || x === 5) && y === 3;
      ctx.fillStyle = isEye ? sprite.eyeColor : y < 3 ? sprite.accentColor : sprite.bodyColor;
      ctx.fillRect(ox + x * px, oy + y * px, px, px);
    }
  }
}
