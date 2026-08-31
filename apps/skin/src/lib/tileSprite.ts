import type { WorldTile } from "./api";
import { cellCenter, HEX_RADIUS } from "./hex";
import { drawOffEyes } from "./sphereEyes";

function fnv1a(str: string): number {
  let h = 2166136261;
  for (let i = 0; i < str.length; i++) {
    h ^= str.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

function tileSeed(x: number, y: number) {
  return fnv1a(`${x},${y}`);
}

function hash01(seed: number, i: number) {
  return ((seed ^ Math.imul(i, 2654435761)) >>> 0) / 4294967295;
}

function drawSolid(ctx: CanvasRenderingContext2D, cx: number, cy: number, seed: number, timeMs: number) {
  const baseR = HEX_RADIUS * 0.44;

  const base = ctx.createRadialGradient(cx - baseR * 0.2, cy - baseR * 0.25, baseR * 0.1, cx, cy, baseR * 1.1);
  base.addColorStop(0, "#5a6574");
  base.addColorStop(0.55, "#3d4550");
  base.addColorStop(1, "#252a32");
  ctx.beginPath();
  ctx.arc(cx, cy, baseR, 0, Math.PI * 2);
  ctx.fillStyle = base;
  ctx.fill();

  for (let i = 0; i < 4; i++) {
    const a = hash01(seed, i) * Math.PI * 2 + timeMs * 0.00002 * (i + 1);
    const dist = baseR * (0.1 + hash01(seed, i + 10) * 0.35);
    const fx = cx + Math.cos(a) * dist;
    const fy = cy + Math.sin(a) * dist;
    const r0 = baseR * (0.22 + hash01(seed, i + 20) * 0.18);
    ctx.beginPath();
    ctx.arc(fx, fy, r0, 0, Math.PI * 2);
    ctx.fillStyle = `rgba(${70 + (i * 9) % 28}, ${78 + (i * 13) % 22}, ${90 + (i * 7) % 18}, 0.45)`;
    ctx.fill();
  }
}

function drawFood(ctx: CanvasRenderingContext2D, cx: number, cy: number, seed: number, timeMs: number) {
  const pulse = 0.94 + Math.sin(timeMs * 0.003 + seed * 0.01) * 0.06;
  const r = HEX_RADIUS * 0.3 * pulse;

  const shadowY = cy + r * 0.52;
  ctx.beginPath();
  ctx.ellipse(cx, shadowY, r * 0.85, r * 0.22, 0, 0, Math.PI * 2);
  ctx.fillStyle = "rgba(0, 0, 0, 0.28)";
  ctx.fill();

  const aura = ctx.createRadialGradient(cx, cy, r * 0.2, cx, cy, r * 1.6);
  aura.addColorStop(0, "rgba(74, 232, 194, 0.18)");
  aura.addColorStop(1, "rgba(74, 232, 194, 0)");
  ctx.fillStyle = aura;
  ctx.beginPath();
  ctx.arc(cx, cy, r * 1.6, 0, Math.PI * 2);
  ctx.fill();

  const body = ctx.createRadialGradient(cx - r * 0.28, cy - r * 0.32, r * 0.08, cx + r * 0.08, cy + r * 0.12, r);
  body.addColorStop(0, "hsl(152 62% 68%)");
  body.addColorStop(0.55, "hsl(168 54% 46%)");
  body.addColorStop(1, "hsl(172 48% 30%)");
  ctx.fillStyle = body;
  ctx.beginPath();
  ctx.arc(cx, cy, r, 0, Math.PI * 2);
  ctx.fill();

  ctx.beginPath();
  ctx.arc(cx - r * 0.22, cy - r * 0.26, r * 0.14, 0, Math.PI * 2);
  ctx.fillStyle = "rgba(230, 255, 245, 0.45)";
  ctx.fill();
}

const BODY_R = HEX_RADIUS * 0.44;

function drawCorpse(ctx: CanvasRenderingContext2D, cx: number, cy: number) {
  const r = BODY_R;
  const x = cx;
  const y = cy;

  const shadowY = y + r * 0.78;
  const ground = ctx.createRadialGradient(x, shadowY, 0, x, shadowY, r * 1.15);
  ground.addColorStop(0, "rgba(40, 18, 6, 0.42)");
  ground.addColorStop(0.45, "rgba(20, 10, 4, 0.16)");
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
  sphere.addColorStop(0, "hsl(26 68% 58%)");
  sphere.addColorStop(0.52, "hsl(22 62% 44%)");
  sphere.addColorStop(1, "hsl(16 52% 30%)");
  ctx.fillStyle = sphere;
  ctx.fill();

  ctx.beginPath();
  ctx.arc(x - r * 0.22, y - r * 0.28, r * 0.11, 0, Math.PI * 2);
  ctx.fillStyle = "rgba(255, 210, 170, 0.22)";
  ctx.fill();

  drawOffEyes(ctx, x, y, r);
}

export function drawTileSprite(ctx: CanvasRenderingContext2D, tile: WorldTile, timeMs: number) {
  const seed = tileSeed(tile.x, tile.y);
  const { x: cx, y: cy } = cellCenter(tile.x, tile.y);

  if (tile.kind === 1) {
    drawSolid(ctx, cx, cy, seed, timeMs);
    return;
  }
  if (tile.kind === 3) {
    drawCorpse(ctx, cx, cy);
    return;
  }
  if (tile.kind === 4) {
    drawFood(ctx, cx, cy, seed, timeMs);
  }
}
