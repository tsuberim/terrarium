import { BODY_R } from "./creatureSprite";
import { eyeWorldPositions } from "./sphereEyes";

export const HIT_LIFE_MS = 750;

function drawLaserBeam(
  ctx: CanvasRenderingContext2D,
  x0: number,
  y0: number,
  x1: number,
  y1: number,
  alpha: number,
  lineWidth: number,
  extend: number,
) {
  const ex = x0 + (x1 - x0) * extend;
  const ey = y0 + (y1 - y0) * extend;

  ctx.save();
  ctx.lineCap = "round";

  ctx.strokeStyle = `rgba(255, 40, 30, ${alpha * 0.28})`;
  ctx.lineWidth = lineWidth * 4;
  ctx.beginPath();
  ctx.moveTo(x0, y0);
  ctx.lineTo(ex, ey);
  ctx.stroke();

  ctx.strokeStyle = `rgba(255, 95, 70, ${alpha * 0.55})`;
  ctx.lineWidth = lineWidth * 1.8;
  ctx.beginPath();
  ctx.moveTo(x0, y0);
  ctx.lineTo(ex, ey);
  ctx.stroke();

  ctx.strokeStyle = `rgba(255, 210, 180, ${alpha * 0.92})`;
  ctx.lineWidth = lineWidth * 0.65;
  ctx.beginPath();
  ctx.moveTo(x0, y0);
  ctx.lineTo(ex, ey);
  ctx.stroke();

  ctx.fillStyle = `rgba(255, 120, 90, ${alpha * 0.85})`;
  ctx.beginPath();
  ctx.arc(x0, y0, lineWidth * 0.9, 0, Math.PI * 2);
  ctx.fill();

  ctx.restore();
}

export function drawHitFx(
  ctx: CanvasRenderingContext2D,
  opts: {
    age: number;
    alpha: number;
    actorCx: number;
    actorCy: number;
    look: number;
    targetX: number;
    targetY: number;
    lineWidth: number;
  },
) {
  const { age, alpha, actorCx, actorCy, look, targetX, targetY, lineWidth } = opts;
  const extend = Math.min(1, age * 3.5);
  const eyes = eyeWorldPositions(actorCx, actorCy, BODY_R, look);

  for (const eye of eyes) {
    drawLaserBeam(ctx, eye.x, eye.y, targetX, targetY, alpha, lineWidth, extend);
  }

  if (age > 0.08) {
    const impact = Math.min(1, (age - 0.08) * 2.5);
    const impactA = alpha * (1 - impact * 0.6);
    ctx.fillStyle = `rgba(255, 80, 50, ${impactA * 0.45})`;
    ctx.beginPath();
    ctx.arc(targetX, targetY, BODY_R * (0.25 + impact * 0.35), 0, Math.PI * 2);
    ctx.fill();
    ctx.strokeStyle = `rgba(255, 180, 140, ${impactA * 0.5})`;
    ctx.lineWidth = lineWidth;
    ctx.beginPath();
    ctx.arc(targetX, targetY, BODY_R * (0.15 + impact * 0.2), 0, Math.PI * 2);
    ctx.stroke();
  }
}

/** 0–1 intensity while actor is firing. */
export function hitFireIntensity(at: number, fxNow: number): number {
  const age = (fxNow - at) / HIT_LIFE_MS;
  if (age >= 1) return 0;
  if (age < 0.08) return age / 0.08;
  return 1 - (age - 0.08) ** 1.4;
}
