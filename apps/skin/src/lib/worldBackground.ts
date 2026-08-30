import { HEX_RADIUS } from "./hex";

/** World pixels per noise sample — rendered low-res, scaled up smooth. */
const CELL = 24;

type Scratch = {
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  pw: number;
  ph: number;
  image: ImageData;
};

let scratch: Scratch | null = null;

function ensureScratch(pw: number, ph: number): Scratch {
  if (scratch && scratch.pw === pw && scratch.ph === ph) return scratch;
  const canvas = document.createElement("canvas");
  canvas.width = pw;
  canvas.height = ph;
  const ctx = canvas.getContext("2d")!;
  scratch = { canvas, ctx, pw, ph, image: ctx.createImageData(pw, ph) };
  return scratch;
}

function hash2(x: number, y: number): number {
  let h = x * 374761393 + y * 668265263;
  h = (h ^ (h >> 13)) >>> 0;
  h = Math.imul(h, 1274126177) >>> 0;
  return (h ^ (h >> 16)) >>> 0;
}

function valueNoise(x: number, y: number): number {
  const x0 = Math.floor(x);
  const y0 = Math.floor(y);
  const fx = x - x0;
  const fy = y - y0;
  const sx = fx * fx * (3 - 2 * fx);
  const sy = fy * fy * (3 - 2 * fy);
  const n00 = hash2(x0, y0) / 4294967295;
  const n10 = hash2(x0 + 1, y0) / 4294967295;
  const n01 = hash2(x0, y0 + 1) / 4294967295;
  const n11 = hash2(x0 + 1, y0 + 1) / 4294967295;
  return (n00 + (n10 - n00) * sx) * (1 - sy) + (n01 + (n11 - n01) * sx) * sy;
}

function fbm(x: number, y: number): number {
  return valueNoise(x, y) * 0.6 + valueNoise(x * 2.1 + 17, y * 2.1 - 9) * 0.4;
}

/** World-space procedural void — pans/zooms with the grid, no block artifacts. */
export function drawWorldBackground(
  ctx: CanvasRenderingContext2D,
  left: number,
  top: number,
  right: number,
  bottom: number,
  timeMs: number,
) {
  const pad = HEX_RADIUS * 3;
  const x0 = left - pad;
  const y0 = top - pad;
  const x1 = right + pad;
  const y1 = bottom + pad;
  const ww = x1 - x0;
  const wh = y1 - y0;

  ctx.fillStyle = "#030508";
  ctx.fillRect(x0, y0, ww, wh);

  const pw = Math.max(1, Math.ceil(ww / CELL));
  const ph = Math.max(1, Math.ceil(wh / CELL));
  const { canvas, ctx: octx, image } = ensureScratch(pw, ph);
  const { data } = image;

  const t = timeMs * 0.000004;
  const freq = 0.012;

  for (let j = 0; j < ph; j++) {
    const wy = y0 + j * CELL;
    for (let i = 0; i < pw; i++) {
      const wx = x0 + i * CELL;
      const n = fbm(wx * freq + t, wy * freq - t * 0.4);
      const k = (j * pw + i) * 4;
      data[k] = 5 + n * 16;
      data[k + 1] = 10 + n * 22;
      data[k + 2] = 16 + n * 28;
      data[k + 3] = 255;
    }
  }

  octx.putImageData(image, 0, 0);

  const prevSmooth = ctx.imageSmoothingEnabled;
  ctx.imageSmoothingEnabled = true;
  ctx.drawImage(canvas, 0, 0, pw, ph, x0, y0, pw * CELL, ph * CELL);
  ctx.imageSmoothingEnabled = prevSmooth;
}
