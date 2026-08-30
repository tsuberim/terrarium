/** Shared live + powered-off eye drawing for creatures and corpse tiles. */

function drawThinPill(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  length: number,
  width: number,
  angle: number,
  fill: string,
) {
  ctx.save();
  ctx.translate(cx, cy);
  ctx.rotate(angle);
  const hw = length / 2;
  const hh = width / 2;
  ctx.beginPath();
  ctx.roundRect(-hw, -hh, length, width, hh);
  ctx.fillStyle = fill;
  ctx.fill();
  ctx.restore();
}

/** Rounded pill aligned with `angle` (radians). */
function drawPill(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  length: number,
  width: number,
  angle: number,
  fill: string,
) {
  if (width < 0.55) {
    ctx.save();
    ctx.translate(cx, cy);
    ctx.rotate(angle);
    ctx.strokeStyle = fill;
    ctx.lineWidth = Math.max(0.85, width * 0.6);
    ctx.lineCap = "round";
    ctx.beginPath();
    ctx.moveTo(-length * 0.5, 0);
    ctx.lineTo(length * 0.5, 0);
    ctx.stroke();
    ctx.restore();
    return;
  }

  ctx.save();
  ctx.translate(cx, cy);
  ctx.rotate(angle);
  const hw = length / 2;
  const hh = width / 2;
  ctx.beginPath();
  ctx.roundRect(-hw, -hh, length, width, hh);
  ctx.fillStyle = fill;
  ctx.fill();
  ctx.restore();
}

/** Closed horizontal lids on the bottom hemisphere — matches corpse tile exactly. */
export function drawOffEyes(ctx: CanvasRenderingContext2D, x: number, y: number, r: number) {
  const look = Math.PI / 2;
  const forward = r * 0.46;
  const halfSpread = r * 0.19;
  const pillLen = r * 0.27;
  const pillW = r * 0.076;
  const fill = "hsla(14, 40%, 22%, 0.75)";

  ctx.save();
  ctx.beginPath();
  ctx.arc(x, y, r * 0.985, 0, Math.PI * 2);
  ctx.clip();
  ctx.translate(x, y);
  ctx.rotate(look);

  for (const side of [-1, 1]) {
    const lx = forward;
    const ly = halfSpread * side;
    const d = Math.hypot(lx, ly) / r;
    const z = Math.sqrt(Math.max(0.04, 1 - d * d));
    const len = pillLen * (0.52 + 0.48 * z);
    const w = pillW * (0.82 + 0.18 * z);
    drawThinPill(ctx, lx, ly, len, w, Math.PI / 2, fill);
  }
  ctx.restore();
}

/** World-space center of each live eye pill. */
export function eyeWorldPositions(
  x: number,
  y: number,
  r: number,
  look: number,
  jitterX = 0,
  jitterY = 0,
): [{ x: number; y: number }, { x: number; y: number }] {
  const cx = x + jitterX;
  const cy = y + jitterY;
  const forward = r * 0.46;
  const halfSpread = r * 0.19;
  const cos = Math.cos(look);
  const sin = Math.sin(look);
  const pts = [-1, 1].map((side) => {
    const lx = forward;
    const ly = halfSpread * side;
    return { x: cx + lx * cos - ly * sin, y: cy + lx * sin + ly * cos };
  });
  return [pts[0], pts[1]];
}

/** Pill eyes on the forward hemisphere. */
export function drawLiveEyes(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  r: number,
  look: number,
  blink: number,
  jitterX = 0,
  jitterY = 0,
  hitFire = 0,
  eatOpen = 0,
) {
  const cx = x + jitterX;
  const cy = y + jitterY;
  const chew = Math.max(0, Math.min(1, eatOpen));
  const eyeScale = 1 + chew * 0.48;
  const pillW = r * 0.088 * (1 - blink * 0.92) * eyeScale;
  const pillLen = r * 0.24 * eyeScale;
  const forward = r * 0.46 * (1 - chew * 0.88);
  const halfSpread = r * 0.19;
  const f = Math.max(0, Math.min(1, hitFire));
  const fill =
    f > 0
      ? `rgba(255, ${Math.round(40 + f * 50)}, ${Math.round(30 + f * 20)}, ${0.88 + f * 0.1})`
      : "rgba(8, 12, 18, 0.9)";

  ctx.save();
  ctx.beginPath();
  ctx.arc(cx, cy, r * 0.985, 0, Math.PI * 2);
  ctx.clip();
  ctx.translate(cx, cy);
  ctx.rotate(look);

  for (const side of [-1, 1]) {
    const lx = forward;
    const ly = halfSpread * side;
    const d = Math.hypot(lx, ly) / r;
    const z = Math.sqrt(Math.max(0.04, 1 - d * d));
    const len = pillLen * (0.52 + 0.48 * z);
    const w = pillW * (0.82 + 0.18 * z);
    drawPill(ctx, lx, ly, len, w, 0, fill);
  }
  ctx.restore();
}

/** Rotate eye frame and morph pills into corpse slits; `t`=1 matches corpse tile exactly. */
export function drawEyesShutdown(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  r: number,
  look: number,
  blink: number,
  jitterX: number,
  jitterY: number,
  t: number,
) {
  const off = Math.max(0, Math.min(1, t));
  if (off >= 1) {
    drawOffEyes(ctx, x, y, r);
    return;
  }
  if (off <= 0) {
    drawLiveEyes(ctx, x, y, r, look, blink, jitterX, jitterY);
    return;
  }

  const u = off;
  const cx = x + jitterX * (1 - u);
  const cy = y + jitterY * (1 - u);
  const offLook = Math.PI / 2;
  const effectiveLook = look + (offLook - look) * u;

  const liveW = r * 0.088 * (1 - blink * 0.92 * (1 - u));
  const liveLen = r * 0.24;
  const offLen = r * 0.27;
  const offW = r * 0.076;
  const forward = r * 0.46;
  const halfSpread = r * 0.19;
  const liveFill = "rgba(8, 12, 18, 0.9)";
  const offFill = "hsla(14, 40%, 22%, 0.75)";

  ctx.save();
  ctx.beginPath();
  ctx.arc(cx, cy, r * 0.985, 0, Math.PI * 2);
  ctx.clip();
  ctx.translate(cx, cy);
  ctx.rotate(effectiveLook);

  for (const side of [-1, 1]) {
    const lx = forward;
    const ly = halfSpread * side;
    const d = Math.hypot(lx, ly) / r;
    const z = Math.sqrt(Math.max(0.04, 1 - d * d));
    const lenLive = liveLen * (0.52 + 0.48 * z);
    const wLive = liveW * (0.82 + 0.18 * z);
    const lenOff = offLen * (0.52 + 0.48 * z);
    const wOff = offW * (0.82 + 0.18 * z);
    const len = lenLive + (lenOff - lenLive) * u;
    const w = wLive + (wOff - wLive) * u;
    const pillAngle = u * (Math.PI / 2);

    if (u >= 0.92) {
      drawThinPill(ctx, lx, ly, lenOff, wOff, Math.PI / 2, offFill);
    } else {
      const fill = u < 0.55 ? liveFill : offFill;
      if (pillAngle > 0.35 || w < 0.55) {
        drawThinPill(ctx, lx, ly, len, w, pillAngle, fill);
      } else {
        drawPill(ctx, lx, ly, len, w, pillAngle, fill);
      }
    }
  }
  ctx.restore();
}
