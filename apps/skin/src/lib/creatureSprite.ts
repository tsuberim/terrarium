import type { Creature } from "./api";
import { HEX_RADIUS, hexPathAt } from "./hex";
import { drawEyesShutdown, drawLiveEyes } from "./sphereEyes";
import { drawEatMouth } from "./eatFx";

export type SpriteMode = "id" | "hash";

export type CreaturePalette = {
  hue: number;
  bodyHighlight: string;
  bodyMid: string;
  bodyShade: string;
  groundShadow: string;
};

const cache = new Map<string, CreaturePalette>();
const SQRT3 = Math.sqrt(3);
export const BODY_R = HEX_RADIUS * 0.44;

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

function hsl(h: number, s: number, l: number, a = 1): string {
  return `hsla(${h % 360} ${s}% ${l}% / ${a})`;
}

export function spriteSeed(creature: Creature, mode: SpriteMode): string {
  if (mode === "hash" && creature.program_hash) return creature.program_hash;
  return creature.id;
}

export function creaturePalette(creature: Creature, mode: SpriteMode): CreaturePalette {
  const seed = spriteSeed(creature, mode);
  const cached = cache.get(seed);
  if (cached) return cached;

  const hue = byte(seed, 99) % 360;
  const palette: CreaturePalette = {
    hue,
    bodyHighlight: hsl(hue, 8, 99),
    bodyMid: hsl(hue, 11, 93),
    bodyShade: hsl(hue, 18, 72),
    groundShadow: hsl(hue, 25, 8, 0.55),
  };
  cache.set(seed, palette);
  return palette;
}

/** @deprecated use creaturePalette */
export function creatureSprite(creature: Creature, mode: SpriteMode) {
  return creaturePalette(creature, mode);
}

export type CreatureAnim = {
  squash: number;
  blink: number;
  eyeJitterX: number;
  eyeJitterY: number;
  leanX: number;
  leanY: number;
};

export const SPAWN_LIFE_MS = 900;
export const DEATH_LIFE_MS = 1050;

function easeOutCubic(t: number) {
  return 1 - (1 - t) ** 3;
}

/** Death motion: quick onset, long gentle settle. */
function deathEase(t: number) {
  return easeOutCubic(Math.max(0, Math.min(1, t)));
}

const WHITE_PALETTE: CreaturePalette = {
  hue: 0,
  bodyHighlight: "hsl(0 0% 99%)",
  bodyMid: "hsl(0 0% 93%)",
  bodyShade: "hsl(0 0% 74%)",
  groundShadow: "rgba(0, 0, 0, 0.55)",
};

const CORPSE_PALETTE: CreaturePalette = {
  hue: 22,
  bodyHighlight: "hsl(26 68% 58%)",
  bodyMid: "hsl(22 62% 44%)",
  bodyShade: "hsl(16 52% 30%)",
  groundShadow: "rgba(40, 18, 6, 0.42)",
};

export type LifeFx = {
  kind: "spawn" | "death";
  /** 0 at start → 1 at end */
  t: number;
};

function easeOutBack(t: number) {
  const c1 = 1.70158;
  const c3 = c1 + 1;
  return 1 + c3 * (t - 1) ** 3 + c1 * (t - 1) ** 2;
}

function easeInOutCubic(t: number) {
  return t < 0.5 ? 4 * t * t * t : 1 - (-2 * t + 2) ** 3 / 2;
}

function parseHsla(s: string): [number, number, number, number] {
  const m = s.match(/hsla?\(\s*([\d.]+)\s+([\d.]+)%\s+([\d.]+)%(?:\s*\/\s*([\d.]+))?\s*\)/);
  if (!m) return [0, 0, 50, 1];
  return [Number(m[1]), Number(m[2]), Number(m[3]), m[4] != null ? Number(m[4]) : 1];
}

function lerpHsla(a: string, b: string, t: number): string {
  const [h1, s1, l1, a1] = parseHsla(a);
  const [h2, s2, l2, a2] = parseHsla(b);
  let dh = h2 - h1;
  if (dh > 180) dh -= 360;
  else if (dh < -180) dh += 360;
  const h = h1 + dh * t;
  const s = s1 + (s2 - s1) * t;
  const l = l1 + (l2 - l1) * t;
  const alpha = a1 + (a2 - a1) * t;
  return `hsla(${h} ${s}% ${l}% / ${alpha})`;
}

function lerpRgba(a: string, b: string, t: number): string {
  const pa = a.match(/rgba?\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)(?:\s*,\s*([\d.]+))?\s*\)/);
  const pb = b.match(/rgba?\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)(?:\s*,\s*([\d.]+))?\s*\)/);
  if (!pa || !pb) return t < 0.5 ? a : b;
  const r = Number(pa[1]) + (Number(pb[1]) - Number(pa[1])) * t;
  const g = Number(pa[2]) + (Number(pb[2]) - Number(pa[2])) * t;
  const bl = Number(pa[3]) + (Number(pb[3]) - Number(pa[3])) * t;
  const alpha = (pa[4] != null ? Number(pa[4]) : 1) + ((pb[4] != null ? Number(pb[4]) : 1) - (pa[4] != null ? Number(pa[4]) : 1)) * t;
  return `rgba(${r}, ${g}, ${bl}, ${alpha})`;
}

export function paletteForHealth(ratio: number): CreaturePalette {
  const t = 1 - Math.max(0, Math.min(1, ratio));
  if (t <= 0) return WHITE_PALETTE;
  if (t >= 1) return CORPSE_PALETTE;
  return blendPaletteTowardCorpse(WHITE_PALETTE, t);
}

export function blendPaletteTowardCorpse(palette: CreaturePalette, t: number): CreaturePalette {
  if (t <= 0) return palette;
  if (t >= 1) return CORPSE_PALETTE;
  return {
    hue: palette.hue + (CORPSE_PALETTE.hue - palette.hue) * t,
    bodyHighlight: lerpHsla(palette.bodyHighlight, CORPSE_PALETTE.bodyHighlight, t),
    bodyMid: lerpHsla(palette.bodyMid, CORPSE_PALETTE.bodyMid, t),
    bodyShade: lerpHsla(palette.bodyShade, CORPSE_PALETTE.bodyShade, t),
    groundShadow: lerpRgba(palette.groundShadow, CORPSE_PALETTE.groundShadow, t),
  };
}

export type LifeModifiers = {
  scale: number;
  alpha: number;
  extraBob: number;
  posT: number;
  eyeOff: number;
  colorT: number;
};

export function lifeModifiers(life?: LifeFx): LifeModifiers {
  if (!life) {
    return { scale: 1, alpha: 1, extraBob: 0, posT: 1, eyeOff: 0, colorT: 0 };
  }
  if (life.kind === "spawn") {
    const raw = Math.min(1, life.t);
    const grow = easeOutBack(raw);
    const fade = easeInOutCubic(raw);
    return {
      scale: grow,
      alpha: fade,
      extraBob: 0,
      posT: fade,
      eyeOff: 0,
      colorT: 0,
    };
  }
  const t = Math.min(1, life.t);
  const colorT = deathEase(t);
  const eyeOff = deathEase(t);
  return {
    scale: 1,
    alpha: 1,
    extraBob: 0,
    posT: 1,
    eyeOff,
    colorT,
  };
}

export function creatureAnim(id: string, now: number, moving: boolean, lookAngle: number): CreatureAnim {
  const seed = fnv1a(id);
  const phase = ((seed % 1000) / 1000) * Math.PI * 2;
  const squash = moving ? 1 + Math.sin(now * 0.012 + phase) * 0.04 : 1 + Math.sin(now * 0.0025 + phase * 0.7) * 0.025;

  const blinkCycle = 3200 + (seed % 1400);
  const blinkT = (now + seed) % blinkCycle;
  const blink = blinkT > blinkCycle - 100 ? (blinkCycle - blinkT) / 100 : 0;

  const drift = now * 0.002 + phase;
  const eyeJitterX = Math.sin(drift) * 0.45 + Math.sin(drift * 0.43 + 1.1) * 0.2;
  const eyeJitterY = Math.sin(drift * 0.51 + 0.7) * 0.38 + Math.cos(drift * 0.29) * 0.16;

  const angle = lookAngle;
  const leanMag = moving ? 2.5 : 0.8;
  return {
    squash,
    blink,
    eyeJitterX,
    eyeJitterY,
    leanX: Math.cos(angle) * leanMag,
    leanY: Math.sin(angle) * leanMag,
  };
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
  mine: boolean,
  lookAngle: number,
  anim: CreatureAnim,
  health?: number,
  maxHealth?: number,
  energy?: { value: number; floor: number; refMax: number },
  life?: LifeFx,
  hitFire = 0,
  eatOpen = 0,
) {
  const look = lookAngle;
  const lifeMod = lifeModifiers(life);
  const isDeath = life?.kind === "death";
  const isSpawn = life?.kind === "spawn";
  const healthRatio =
    health !== undefined && maxHealth !== undefined && maxHealth > 0
      ? Math.max(0, Math.min(1, health / maxHealth))
      : 1;
  let drawPalette = paletteForHealth(healthRatio);
  if (isDeath) drawPalette = blendPaletteTowardCorpse(drawPalette, lifeMod.colorT);
  const x = isDeath || isSpawn ? cx : cx + anim.leanX;
  const y = isDeath || isSpawn ? cy : cy + anim.leanY * 0.3 + lifeMod.extraBob;
  const r = BODY_R * (isDeath || isSpawn ? 1 : anim.squash) * lifeMod.scale;

  ctx.save();
  ctx.globalAlpha = lifeMod.alpha;

  const shadowY = y + r * 0.78;
  const ground = ctx.createRadialGradient(x, shadowY, 0, x, shadowY, r * 1.15);
  ground.addColorStop(0, drawPalette.groundShadow);
  ground.addColorStop(0.45, "rgba(0, 0, 0, 0.18)");
  ground.addColorStop(1, "rgba(0, 0, 0, 0)");
  ctx.fillStyle = ground;
  ctx.beginPath();
  ctx.ellipse(x, shadowY, r * 0.92, r * 0.26, 0, 0, Math.PI * 2);
  ctx.fill();

  ctx.beginPath();
  ctx.arc(x, y, r, 0, Math.PI * 2);
  const sphere = ctx.createRadialGradient(
    x - r * 0.38,
    y - r * 0.42,
    r * 0.06,
    x + r * 0.12,
    y + r * 0.18,
    r * 1.08,
  );
  sphere.addColorStop(0, drawPalette.bodyHighlight);
  sphere.addColorStop(0.52, drawPalette.bodyMid);
  sphere.addColorStop(1, drawPalette.bodyShade);
  ctx.fillStyle = sphere;
  ctx.fill();

  const specAlpha = isDeath ? 0.22 * lifeMod.colorT + 0.55 * (1 - lifeMod.colorT) : 0.35 + healthRatio * 0.2;
  ctx.beginPath();
  ctx.arc(x - r * 0.22, y - r * 0.28, r * 0.11, 0, Math.PI * 2);
  ctx.fillStyle = isDeath
    ? `rgba(255, 210, 170, ${specAlpha})`
    : `rgba(255, 255, 255, ${specAlpha})`;
  ctx.fill();

  if (isDeath) {
    drawEyesShutdown(ctx, x, y, r, look, anim.blink, anim.eyeJitterX, anim.eyeJitterY, lifeMod.eyeOff);
  } else {
    drawLiveEyes(ctx, x, y, r, look, anim.blink, anim.eyeJitterX, anim.eyeJitterY, hitFire, eatOpen);
    if (eatOpen > 0.005) drawEatMouth(ctx, x, y, r, look, eatOpen);
  }

  if (mine && energy) {
    const barBase = cy + HEX_RADIUS * 0.58;
    const span = Math.max(energy.refMax - energy.floor, 1);
    const ratio = Math.max(0, Math.min(1, (energy.value - energy.floor) / span));
    const color =
      ratio <= 0.12 ? "rgba(232, 100, 90, 0.9)" : ratio <= 0.35 ? "rgba(232, 168, 74, 0.9)" : "rgba(74, 232, 194, 0.95)";
    drawStatusBar(ctx, cx, barBase, ratio, color);
  }

  ctx.restore();

  if (mine) {
    hexPathAt(ctx, cx, cy, HEX_RADIUS * 0.92);
    ctx.strokeStyle = "rgba(74, 232, 194, 0.18)";
    ctx.lineWidth = 1;
    ctx.stroke();
  }
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
