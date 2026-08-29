import type { Creature } from "./api";
import { HEX_RADIUS, hexPathAt } from "./hex";

export type SpriteMode = "id" | "hash";

/** Spacing of the 7-hex sprite lattice relative to the cell. */
const SPRITE_SCALE = 0.41;
const CELL_CLIP = 0.94;
/** Room for status bars along the bottom flat edge. */
const BAR_RESERVE = 3;

/** East-facing critter: center + ring (+ optional seed accent on NW). */
const LAYOUT: { q: number; r: number; tone: PixelTone }[] = [
  { q: 0, r: 0, tone: "body" },
  { q: 1, r: -1, tone: "eye" },
  { q: 0, r: 1, tone: "eye" },
  { q: 1, r: 0, tone: "accent" },
  { q: -1, r: 0, tone: "body" },
  { q: 0, r: -1, tone: "body" },
  { q: -1, r: 1, tone: "body" },
];

export type PixelTone = "accent" | "body" | "eye";

export type HexPixel = {
  q: number;
  r: number;
  tone: PixelTone;
};

export type CreatureSprite = {
  pixels: HexPixel[];
  bodyColor: string;
  accentColor: string;
  eyeColor: string;
};

const cache = new Map<string, CreatureSprite>();

const SQRT3 = Math.sqrt(3);

/** Mini-hex radius so neighbors overlap into a solid blob. */
function pixelRadius() {
  return HEX_RADIUS * SPRITE_SCALE * (SQRT3 / 2) * 1.07;
}

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

export function rotateAxial(q: number, r: number, steps: number): [number, number] {
  let aq = q;
  let ar = r;
  const n = ((steps % 6) + 6) % 6;
  for (let i = 0; i < n; i++) {
    const nq = -ar;
    const nr = aq + ar;
    aq = nq;
    ar = nr;
  }
  return [aq, ar];
}

function latticeCoord(q: number, r: number) {
  return {
    x: HEX_RADIUS * SPRITE_SCALE * SQRT3 * (q + r / 2),
    y: HEX_RADIUS * SPRITE_SCALE * (1.5 * r),
  };
}

/** Center the mini-hex blob in the cell, leaving a sliver for status bars. */
function spriteCenterOffset(pixels: HexPixel[], dir: number) {
  const pr = pixelRadius();
  const hw = pr * (SQRT3 / 2);
  const hh = pr;
  let minX = Infinity;
  let maxX = -Infinity;
  let minY = Infinity;
  let maxY = -Infinity;
  for (const pixel of pixels) {
    const [q, r] = rotateAxial(pixel.q, pixel.r, dir);
    const { x, y } = latticeCoord(q, r);
    minX = Math.min(minX, x - hw);
    maxX = Math.max(maxX, x + hw);
    minY = Math.min(minY, y - hh);
    maxY = Math.max(maxY, y + hh);
  }
  return {
    ox: -(minX + maxX) / 2,
    oy: -(minY + maxY) / 2 - BAR_RESERVE / 2,
  };
}

function latticePoint(cx: number, cy: number, q: number, r: number, offset: { ox: number; oy: number }) {
  const { x, y } = latticeCoord(q, r);
  return { x: cx + x + offset.ox, y: cy + y + offset.oy };
}

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

  const pixels = LAYOUT.map(({ q, r, tone }) => {
    if (q === 0 && r === -1) {
      return { q, r, tone: (byte(seed, 31) & 1) === 0 ? ("accent" as const) : ("body" as const) };
    }
    return { q, r, tone };
  });

  const hue = byte(seed, 99) % 360;
  const sprite: CreatureSprite = {
    pixels,
    bodyColor: hsl(hue, 52, 44),
    accentColor: hsl(hue + 36, 62, 58),
    eyeColor: hsl(hue + 180, 15, 94),
  };
  cache.set(seed, sprite);
  return sprite;
}

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

function toneColor(sprite: CreatureSprite, tone: PixelTone) {
  if (tone === "eye") return sprite.eyeColor;
  if (tone === "accent") return sprite.accentColor;
  return sprite.bodyColor;
}

function drawStatusBar(ctx: CanvasRenderingContext2D, cx: number, y: number, ratio: number, color: string) {
  const w = HEX_RADIUS * SQRT3 * 0.72;
  const h = 1.25;
  const x = cx - w / 2;
  ctx.fillStyle = "rgba(0, 0, 0, 0.55)";
  ctx.fillRect(x, y, w, h);
  if (ratio > 0) {
    ctx.fillStyle = color;
    ctx.fillRect(x, y, w * ratio, h);
  }
}

export function drawCreatureSprite(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  sprite: CreatureSprite,
  mine: boolean,
  health?: number,
  maxHealth?: number,
  facing: number | null = null,
  energy?: { value: number; floor: number; refMax: number },
) {
  const dir = facing ?? 0;
  const pr = pixelRadius();
  const offset = spriteCenterOffset(sprite.pixels, dir);

  ctx.save();
  hexPathAt(ctx, cx, cy, HEX_RADIUS * CELL_CLIP);
  ctx.clip();

  // Draw back pixels first, eyes last.
  const order: PixelTone[] = ["body", "accent", "eye"];
  for (const tone of order) {
    for (const pixel of sprite.pixels) {
      if (pixel.tone !== tone) continue;
      const [q, r] = rotateAxial(pixel.q, pixel.r, dir);
      const { x, y } = latticePoint(cx, cy, q, r, offset);
      hexPathAt(ctx, x, y, pr);
      ctx.fillStyle = toneColor(sprite, pixel.tone);
      ctx.fill();
    }
  }

  const barBase = cy + HEX_RADIUS * 0.58;
  if (health !== undefined && maxHealth !== undefined && maxHealth > 0) {
    const ratio = Math.max(0, Math.min(1, health / maxHealth));
    const color =
      ratio >= 1 ? "rgba(74, 232, 194, 0.85)" : ratio > 0.35 ? "rgba(255, 90, 60, 0.9)" : "rgba(255, 48, 48, 0.95)";
    drawStatusBar(ctx, cx, barBase, ratio, color);
  }

  if (mine && energy) {
    const span = Math.max(energy.refMax - energy.floor, 1);
    const ratio = Math.max(0, Math.min(1, (energy.value - energy.floor) / span));
    const color =
      ratio <= 0.12 ? "rgba(232, 100, 90, 0.9)" : ratio <= 0.35 ? "rgba(232, 168, 74, 0.9)" : "rgba(74, 232, 194, 0.95)";
    drawStatusBar(ctx, cx, barBase + 2, ratio, color);
  }

  ctx.restore();

  if (mine) {
    hexPathAt(ctx, cx, cy, HEX_RADIUS * CELL_CLIP);
    ctx.strokeStyle = "rgba(74, 232, 194, 0.22)";
    ctx.lineWidth = 1;
    ctx.stroke();
  }
}
