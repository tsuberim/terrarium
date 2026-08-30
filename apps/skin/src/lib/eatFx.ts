export const EAT_LIFE_MS = 820;

/** Local face coords (same frame as eyes): forward along +x, lateral along +y. */
export const MOUTH_RIM = 0.93;

function mouthLocal(r: number): { lx: number; ly: number; z: number } {
  const lx = r * MOUTH_RIM;
  const ly = 0;
  const d = lx / r;
  const z = Math.sqrt(Math.max(0.04, 1 - d * d));
  return { lx, ly, z };
}

function easeOutCubic(t: number) {
  return 1 - (1 - t) ** 3;
}

function easeInCubic(t: number) {
  return t ** 3;
}

function easeInOutCubic(t: number) {
  return t < 0.5 ? 4 * t * t * t : 1 - (-2 * t + 2) ** 3 / 2;
}

/** Mouth opens then closes: 0 → 1 → 0 over normalized age. */
export function mouthEnvelope(age: number): number {
  if (age < 0.36) return easeOutCubic(age / 0.36);
  return Math.max(0, 1 - easeInCubic((age - 0.36) / 0.64));
}

export function foodTravelT(age: number): number {
  return easeInOutCubic(Math.max(0, Math.min(1, (age - 0.1) / 0.62)));
}

/** World-space mouth center on the forward face, below the eye line. */
export function mouthWorldPosition(
  x: number,
  y: number,
  r: number,
  look: number,
): { x: number; y: number } {
  const { lx, ly } = mouthLocal(r);
  const cos = Math.cos(look);
  const sin = Math.sin(look);
  return {
    x: x + lx * cos - ly * sin,
    y: y + lx * sin + ly * cos,
  };
}

/** Black mouth ellipse on the forward rim; scales with `open`, clipped to body. */
export function drawEatMouth(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  r: number,
  look: number,
  open: number,
) {
  if (open <= 0.005) return;

  const { lx, ly, z } = mouthLocal(r);
  const mouthR = r * 0.72 * open * (0.45 + 0.55 * z);

  ctx.save();
  ctx.beginPath();
  ctx.arc(x, y, r * 0.985, 0, Math.PI * 2);
  ctx.clip();
  ctx.translate(x, y);
  ctx.rotate(look);
  ctx.translate(lx, ly);
  ctx.fillStyle = "#030305";
  ctx.beginPath();
  ctx.arc(0, 0, mouthR, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();
}

function drawFoodBlob(ctx: CanvasRenderingContext2D, cx: number, cy: number, r: number, alpha: number) {
  ctx.save();
  ctx.globalAlpha *= alpha;
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
  ctx.restore();
}

function drawCorpseBlob(ctx: CanvasRenderingContext2D, cx: number, cy: number, r: number, alpha: number) {
  ctx.save();
  ctx.globalAlpha *= alpha;
  const sphere = ctx.createRadialGradient(
    cx - r * 0.38,
    cy - r * 0.42,
    r * 0.06,
    cx + r * 0.12,
    cy + r * 0.18,
    r * 1.08,
  );
  sphere.addColorStop(0, "hsl(26 68% 58%)");
  sphere.addColorStop(0.52, "hsl(22 62% 44%)");
  sphere.addColorStop(1, "hsl(16 52% 30%)");
  ctx.fillStyle = sphere;
  ctx.beginPath();
  ctx.arc(cx, cy, r, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();
}

export function drawEatFx(
  ctx: CanvasRenderingContext2D,
  opts: {
    age: number;
    alpha: number;
    actorCx: number;
    actorCy: number;
    look: number;
    foodCx: number;
    foodCy: number;
    tileKind: number;
    bodyR: number;
  },
) {
  const { age, alpha, actorCx, actorCy, look, foodCx, foodCy, tileKind, bodyR } = opts;
  const mouthOpen = mouthEnvelope(age);
  if (mouthOpen <= 0.001 && foodTravelT(age) >= 0.99) return;

  const mouth = mouthWorldPosition(actorCx, actorCy, bodyR, look);

  const travel = foodTravelT(age);
  const biteX = foodCx + (mouth.x - foodCx) * travel;
  const biteY = foodCy + (mouth.y - foodCy) * travel;
  const biteBaseR = bodyR;
  const biteBaseFoodR = bodyR * 0.68;
  const biteBaseRKind = tileKind === 3 ? biteBaseR : biteBaseFoodR;
  const biteR = biteBaseRKind * (1 - travel * 0.82);
  const biteAlpha = alpha * (1 - travel * 0.92);

  if (biteAlpha > 0.02 && biteR > 0.4) {
    if (tileKind === 3) drawCorpseBlob(ctx, biteX, biteY, biteR, biteAlpha);
    else drawFoodBlob(ctx, biteX, biteY, biteR, biteAlpha);
  }
}
