/** Matches sim `ENERGY_SCALE` — one display unit. */
export const GLIM_SCALE = 100_000;

export const GLIM_NAME = "glim";
export const GLIM_NAME_PLURAL = "glims";

/** Fixed-width numeric field width (glim units, incl. decimal). */
export const GLIM_VALUE_WIDTH_CH = 6;

export function glimCount(raw: number): number {
  return raw / GLIM_SCALE;
}

function pluralUnit(count: number): string {
  return Math.abs(count) === 1 ? GLIM_NAME : GLIM_NAME_PLURAL;
}

/** Numeric glim amount with fixed precision — stable width in UI. */
export function formatGlimValue(raw: number): string {
  const n = glimCount(raw);
  const abs = Math.abs(n);
  if (abs >= 1000) {
    return `${(n / 1000).toFixed(1)}k`;
  }
  return n.toFixed(1);
}

/** Plain text, e.g. `10.0 glims` — use `GlimAmount` in React when you want the icon. */
export function formatGlimString(raw: number): string {
  const count = glimCount(raw);
  return `${formatGlimValue(raw)} ${pluralUnit(count)}`;
}

/** Plain text with icon prefix for non-React surfaces (status strings). */
export function formatGlimLabel(raw: number): string {
  return `◆ ${formatGlimString(raw)}`;
}
