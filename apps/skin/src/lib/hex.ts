/** Axial hex layout (pointy-top). Sim `x`/`y` are q/r. */

export const HEX_RADIUS = 8;
const SQRT3 = Math.sqrt(3);

export function axialToPixel(q: number, r: number) {
  const x = HEX_RADIUS * SQRT3 * (q + r / 2);
  const y = HEX_RADIUS * (1.5 * r);
  return { x, y };
}

function axialRound(fq: number, fr: number) {
  const fs = -fq - fr;
  let q = Math.round(fq);
  let r = Math.round(fr);
  const s = Math.round(fs);
  const dq = Math.abs(q - fq);
  const dr = Math.abs(r - fr);
  const ds = Math.abs(s - fs);
  if (dq > dr && dq > ds) q = -r - s;
  else if (dr > ds) r = -q - s;
  return { q, r };
}

export function pixelToAxial(x: number, y: number) {
  const fq = (SQRT3 / 3 * x - y / 3) / HEX_RADIUS;
  const fr = ((2 / 3) * y) / HEX_RADIUS;
  return axialRound(fq, fr);
}

/** Fractional axial — for viewport bounds (don't round). */
export function pixelToAxialFloat(x: number, y: number) {
  return {
    q: (SQRT3 / 3 * x - y / 3) / HEX_RADIUS,
    r: ((2 / 3) * y) / HEX_RADIUS,
  };
}

/** q/r iteration range covering a pixel viewport (all four corners). */
export function visibleHexRange(
  left: number,
  top: number,
  right: number,
  bottom: number,
  pad = 2,
) {
  const corners = [
    pixelToAxialFloat(left, top),
    pixelToAxialFloat(right, top),
    pixelToAxialFloat(left, bottom),
    pixelToAxialFloat(right, bottom),
  ];
  let qMin = Infinity;
  let qMax = -Infinity;
  let rMin = Infinity;
  let rMax = -Infinity;
  for (const c of corners) {
    qMin = Math.min(qMin, c.q);
    qMax = Math.max(qMax, c.q);
    rMin = Math.min(rMin, c.r);
    rMax = Math.max(rMax, c.r);
  }
  return {
    qMin: Math.floor(qMin) - pad,
    qMax: Math.ceil(qMax) + pad,
    rMin: Math.floor(rMin) - pad,
    rMax: Math.ceil(rMax) + pad,
  };
}

export function hexIntersectsViewport(
  q: number,
  r: number,
  left: number,
  top: number,
  right: number,
  bottom: number,
) {
  const corners = hexCorners(q, r);
  let minX = Infinity;
  let maxX = -Infinity;
  let minY = Infinity;
  let maxY = -Infinity;
  for (const p of corners) {
    minX = Math.min(minX, p.x);
    maxX = Math.max(maxX, p.x);
    minY = Math.min(minY, p.y);
    maxY = Math.max(maxY, p.y);
  }
  return !(maxX < left || minX > right || maxY < top || minY > bottom);
}

export function hexDistance(q1: number, r1: number, q2: number, r2: number) {
  const dq = Math.abs(q1 - q2);
  const dr = Math.abs(r1 - r2);
  const ds = Math.abs(q1 + r1 - q2 - r2);
  return (dq + dr + ds) / 2;
}

export function hexPath(ctx: CanvasRenderingContext2D, q: number, r: number) {
  const { x: cx, y: cy } = axialToPixel(q, r);
  hexPathAt(ctx, cx, cy, HEX_RADIUS);
}

/** Pointy-top hex centered at pixel `(cx, cy)`. */
export function hexPathAt(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  radius = HEX_RADIUS,
) {
  const corners = hexCornersAt(cx, cy, radius);
  ctx.beginPath();
  ctx.moveTo(corners[0].x, corners[0].y);
  for (let i = 1; i < 6; i++) ctx.lineTo(corners[i].x, corners[i].y);
  ctx.closePath();
}

export function hexCorners(q: number, r: number) {
  const { x: cx, y: cy } = axialToPixel(q, r);
  return hexCornersAt(cx, cy, HEX_RADIUS);
}

function hexCornersAt(cx: number, cy: number, radius: number) {
  const pts: { x: number; y: number }[] = [];
  for (let i = 0; i < 6; i++) {
    const angle = (Math.PI / 180) * (60 * i - 30);
    pts.push({
      x: cx + radius * Math.cos(angle),
      y: cy + radius * Math.sin(angle),
    });
  }
  return pts;
}

/** All axial cells within hex vision radius `range` (inclusive). */
export function hexDisk(q: number, r: number, range: number) {
  const cells: { q: number; r: number }[] = [];
  for (let dq = -range; dq <= range; dq++) {
    const rMin = Math.max(-range, -dq - range);
    const rMax = Math.min(range, -dq + range);
    for (let dr = rMin; dr <= rMax; dr++) {
      if (hexDistance(0, 0, dq, dr) <= range) {
        cells.push({ q: q + dq, r: r + dr });
      }
    }
  }
  return cells;
}

export function cellCenter(q: number, r: number) {
  return axialToPixel(q, r);
}
