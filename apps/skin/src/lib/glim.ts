/** Matches kernel `ENERGY_SCALE` — one display unit. */
export const GLIM_SCALE = 100_000;

export const GLIM_NAME = "glim";
export const GLIM_NAME_PLURAL = "glims";

export function glimCount(raw: number): number {
  return raw / GLIM_SCALE;
}

function pluralUnit(count: number): string {
  return Math.abs(count) === 1 ? GLIM_NAME : GLIM_NAME_PLURAL;
}

/** Numeric glim amount only, e.g. `10`, `1.5`, `100`. */
export function formatGlimValue(raw: number): string {
  const n = glimCount(raw);
  const abs = Math.abs(n);
  if (abs >= 10_000) {
    return `${(n / 1000).toFixed(0)}k`;
  }
  if (abs >= 1000) {
    return (n / 1000).toFixed(1).replace(/\.0$/, "") + "k";
  }
  if (abs >= 100) {
    return Math.round(n).toString();
  }
  if (abs >= 10) {
    return n.toFixed(1).replace(/\.0$/, "");
  }
  if (abs >= 1) {
    return n.toFixed(1).replace(/\.0$/, "");
  }
  if (abs === 0) {
    return "0";
  }
  return n.toFixed(2).replace(/0+$/, "").replace(/\.$/, "");
}

/** Plain text, e.g. `10 glims` — use `GlimAmount` in React when you want the icon. */
export function formatGlimString(raw: number): string {
  const count = glimCount(raw);
  return `${formatGlimValue(raw)} ${pluralUnit(count)}`;
}

/** Plain text with icon prefix for non-React surfaces (status strings). */
export function formatGlimLabel(raw: number): string {
  return `◆ ${formatGlimString(raw)}`;
}
