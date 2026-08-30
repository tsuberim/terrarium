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

/** Pointy-top hex corners for cell `(q, r)`. */
export function hexCorners(q: number, r: number) {
  const { x: cx, y: cy } = axialToPixel(q, r);
  return hexCornersAt(cx, cy, HEX_RADIUS);
}

function rotateAround(
  ox: number,
  oy: number,
  px: number,
  py: number,
  angle: number,
): { x: number; y: number } {
  const dx = px - ox;
  const dy = py - oy;
  const c = Math.cos(angle);
  const s = Math.sin(angle);
  return { x: ox + dx * c - dy * s, y: oy + dx * s + dy * c };
}

/**
 * FOV outline matching kernel `sense`: hex-range disk clipped to frontal arc.
 * Optional `lookAngle` rotates the shape for smooth facing animation.
 */
export function fovOutlinePoints(
  cq: number,
  cr: number,
  facing: number,
  range: number,
  halfArc: number,
  lookAngle?: number,
): { x: number; y: number }[] {
  const origin = cellCenter(cq, cr);
  const visible = hexDisk(cq, cr, range).filter((cell) =>
    hexInSense(cq, cr, facing, cell.q, cell.r, range, halfArc),
  );
  const visKey = new Set(visible.map((cell) => `${cell.q},${cell.r}`));

  const edgePts: { x: number; y: number }[] = [];
  for (const cell of visible) {
    const corners = hexCorners(cell.q, cell.r);
    for (let d = 0; d < 6; d++) {
      const [dq, dr] = NEIGHBOR_OFFSETS[d];
      if (visKey.has(`${cell.q + dq},${cell.r + dr}`)) continue;
      edgePts.push(corners[d], corners[(d + 1) % 6]);
    }
  }

  if (edgePts.length === 0) return [origin];

  const deduped: { x: number; y: number; a: number }[] = [];
  for (const p of edgePts) {
    if (!deduped.some((q) => Math.hypot(q.x - p.x, q.y - p.y) < 0.5)) {
      deduped.push({ ...p, a: Math.atan2(p.y - origin.y, p.x - origin.x) });
    }
  }
  deduped.sort((a, b) => a.a - b.a);

  const rot = lookAngle != null ? lookAngle - facingAngle(facing) : 0;
  const outline = deduped.map((p) => (rot ? rotateAround(origin.x, origin.y, p.x, p.y, rot) : p));
  return [origin, ...outline];
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

/** Pixel radius covering all hex cells within axial range `range` from origin. */
export function hexRangePixelRadius(range: number) {
  let max = HEX_RADIUS;
  for (const cell of hexDisk(0, 0, range)) {
    const { x, y } = axialToPixel(cell.q, cell.r);
    max = Math.max(max, Math.hypot(x, y) + HEX_RADIUS);
  }
  return max;
}

const NEIGHBOR_OFFSETS: [number, number][] = [
  [1, 0],
  [1, -1],
  [0, -1],
  [-1, 0],
  [-1, 1],
  [0, 1],
];

function dirOfOffset(dq: number, dr: number): number | null {
  for (let d = 0; d < 6; d++) {
    if (NEIGHBOR_OFFSETS[d][0] === dq && NEIGHBOR_OFFSETS[d][1] === dr) return d;
  }
  return null;
}

function directionToward(dq: number, dr: number): number | null {
  if (dq === 0 && dr === 0) return null;
  const qf = dq;
  const rf = dr;
  const sf = -qf - rf;
  let rq = Math.round(qf);
  let rr = Math.round(rf);
  let rs = Math.round(sf);
  const qDiff = Math.abs(rq - qf);
  const rDiff = Math.abs(rr - rf);
  const sDiff = Math.abs(rs - sf);
  if (qDiff > rDiff && qDiff > sDiff) rq = -rr - rs;
  else if (rDiff > sDiff) rr = -rq - rs;
  else rs = -rq - rr;
  return dirOfOffset(rq, rr);
}

function relativeBearing(facing: number, dq: number, dr: number): number | null {
  const target = directionToward(dq, dr);
  if (target == null) return null;
  const diff = (((target - facing) % 6) + 6) % 6;
  return diff > 3 ? diff - 6 : diff;
}

/** True when target is within sense range and frontal arc (matches kernel sense). */
export function hexInSense(
  cq: number,
  cr: number,
  facing: number,
  tq: number,
  tr: number,
  range: number,
  halfArc: number,
): boolean {
  if (hexDistance(cq, cr, tq, tr) > range) return false;
  return hexInFov(cq, cr, facing, tq, tr, halfArc);
}
/** True when target cell is within frontal arc ±halfArc hex steps from facing. */
export function hexInFov(
  cq: number,
  cr: number,
  facing: number,
  tq: number,
  tr: number,
  halfArc: number,
): boolean {
  const dq = tq - cq;
  const dr = tr - cr;
  if (dq === 0 && dr === 0) return true;
  const bearing = relativeBearing(facing, dq, dr);
  return bearing != null && Math.abs(bearing) <= halfArc;
}

/** Radians from creature center toward hex direction 0–5. */
export function facingAngle(facing: number) {
  const [dq, dr] = NEIGHBOR_OFFSETS[((facing % 6) + 6) % 6];
  const origin = axialToPixel(0, 0);
  const target = axialToPixel(dq, dr);
  return Math.atan2(target.y - origin.y, target.x - origin.x);
}

/** Shortest-path angle interpolation. */
export function lerpAngle(from: number, to: number, t: number) {
  let diff = to - from;
  if (diff > Math.PI) diff -= Math.PI * 2;
  else if (diff < -Math.PI) diff += Math.PI * 2;
  return from + diff * t;
}
