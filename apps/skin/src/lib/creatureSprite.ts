import type { Creature } from "./api";
import { HEX_RADIUS, hexPathAt } from "./hex";

export type SpriteMode = "id" | "hash";

/** Hex-shaped 6×6 critter canvas (pointy-top silhouette). */
const W = 6;
const H = 6;

/** Pointy-top hex rows inside the sprite grid. */
const HEX_MASK = [
  [0, 0, 1, 1, 0, 0],
  [0, 1, 1, 1, 1, 0],
  [1, 1, 1, 1, 1, 1],
  [1, 1, 1, 1, 1, 1],
  [0, 1, 1, 1, 1, 0],
  [0, 0, 1, 1, 0, 0],
] as const;

export type CreatureSprite = {
  body: boolean[];
  bodyColor: string;
  accentColor: string;
  eyeColor: string;
  /** Hex direction 0–5 the sprite faces (E … SE). */
  facing: number;
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

function inMask(x: number, y: number) {
  return y >= 0 && y < H && x >= 0 && x < W && HEX_MASK[y][x] === 1;
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

  const body: boolean[] = new Array(W * H).fill(false);
  for (let y = 0; y < H; y++) {
    for (let x = 0; x < 3; x++) {
      if (!inMask(x, y)) continue;
      const bit = (byte(seed, y * 3 + x) >> (x % 3)) & 1;
      if (!bit) continue;
      body[y * W + x] = true;
      const mirror = W - 1 - x;
      if (mirror !== x && inMask(mirror, y)) body[y * W + mirror] = true;
    }
  }

  // Belly fill + eyes — kept inside the hex mask.
  for (let y = 2; y < 5; y++) {
    for (let x = 1; x < 5; x++) {
      if (inMask(x, y)) body[y * W + x] = true;
    }
  }
  if (inMask(1, 1)) body[1 * W + 1] = true;
  if (inMask(4, 1)) body[1 * W + 4] = true;
  body[1 * W + 1] = false;
  body[1 * W + 4] = false;
  if (inMask(1, 2)) body[2 * W + 1] = true;
  if (inMask(4, 2)) body[2 * W + 4] = true;

  const hue = byte(seed, 99) % 360;
  const facing = byte(seed, 77) % 6;
  const sprite: CreatureSprite = {
    body,
    bodyColor: hsl(hue, 58, 48),
    accentColor: hsl(hue + 40, 70, 62),
    eyeColor: hsl(hue + 180, 20, 92),
    facing,
  };
  cache.set(seed, sprite);
  return sprite;
}

/** Map axial step to hex direction index (E=0 … SE=5). */
export function facingFromDelta(dq: number, dr: number): number | null {
  const dirs: [number, number][] = [
    [1, 0],
    [1, -1],
    [0, -1],
    [-1, 0],
    [-1, 1],
    [0, 1],
  ];
  for (let i = 0; i < dirs.length; i++) {
    if (dirs[i][0] === dq && dirs[i][1] === dr) return i;
  }
  return null;
}

/** Pixel size chosen so the sprite fits the inner hex (≈12px). */
export function spritePixelSize() {
  return 2;
}

export function drawCreatureSprite(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  sprite: CreatureSprite,
  mine: boolean,
  health?: number,
  maxHealth?: number,
  facing?: number | null,
) {
  const px = spritePixelSize();
  const spanW = W * px;
  const spanH = H * px;
  const ox = Math.round(cx - spanW / 2);
  const oy = Math.round(cy - spanH / 2 - px * 0.5);

  ctx.save();
  hexPathAt(ctx, cx, cy, HEX_RADIUS * 0.92);
  ctx.fillStyle = mine ? "rgba(74, 232, 194, 0.1)" : "rgba(123, 109, 255, 0.1)";
  ctx.fill();
  hexPathAt(ctx, cx, cy, HEX_RADIUS * 0.92);
  ctx.clip();

  const dir = facing ?? sprite.facing;
  const eyeShift =
    dir === 0 ? 1 : dir === 3 ? -1 : dir === 1 || dir === 5 ? 0 : 0;

  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      if (!sprite.body[y * W + x]) continue;
      let ex = x;
      if (eyeShift > 0 && x >= 3) ex = Math.min(W - 1, x + 1);
      if (eyeShift < 0 && x <= 2) ex = Math.max(0, x - 1);
      const isEye = y === 2 && (ex === 1 || ex === 4);
      ctx.fillStyle = isEye ? sprite.eyeColor : y < 2 ? sprite.accentColor : sprite.bodyColor;
      ctx.fillRect(ox + x * px, oy + y * px, px, px);
    }
  }
  ctx.restore();

  if (health !== undefined && maxHealth !== undefined && maxHealth > 0) {
    const barW = HEX_RADIUS * SQRT3 * 0.82;
    const barH = 2;
    const bx = cx - barW / 2;
    const by = cy + HEX_RADIUS * 0.42;
    const ratio = Math.max(0, Math.min(1, health / maxHealth));
    ctx.fillStyle = "rgba(0, 0, 0, 0.45)";
    ctx.fillRect(bx, by, barW, barH);
    ctx.fillStyle =
      ratio >= 1 ? "rgba(74, 232, 194, 0.75)" : ratio > 0.35 ? "rgba(255, 90, 60, 0.85)" : "rgba(255, 48, 48, 0.9)";
    ctx.fillRect(bx, by, barW * ratio, barH);
  }
}

const SQRT3 = Math.sqrt(3);
