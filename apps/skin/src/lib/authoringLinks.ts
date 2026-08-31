/** GitHub repo imported by Replit. */
export const GITHUB_REPO = "tsuberim/terrarium";

const DEFAULT_API_BASE = "https://terrarium-506917.web.app/api";
const DEFAULT_DOCS_BASE = "https://terrarium.mintlify.app";
const DEFAULT_ENERGY = 10_000_000;

export const docsBase =
  (import.meta.env.VITE_DOCS_URL as string | undefined)?.replace(/\/$/, "") || DEFAULT_DOCS_BASE;

export const authorLinks = {
  replit: `https://replit.com/github.com/${GITHUB_REPO}`,
  docs: `${docsBase}/getting-started/replit`,
  docsHome: docsBase,
} as const;

export function envFileContent(opts: {
  apiBase: string;
  x: number;
  y: number;
  energy?: number;
}) {
  const base = opts.apiBase.replace(/\/$/, "");
  const energy = opts.energy ?? DEFAULT_ENERGY;
  return [
    `TERRARIUM_API_BASE=${base}`,
    "TERRARIUM_API_KEY=",
    `TERRARIUM_X=${opts.x}`,
    `TERRARIUM_Y=${opts.y}`,
    `TERRARIUM_ENERGY=${energy}`,
  ].join("\n");
}

export function resolveApiBase(apiRoot: string) {
  const trimmed = apiRoot.replace(/\/$/, "");
  if (trimmed && trimmed !== "/api") return trimmed;
  if (typeof window !== "undefined" && window.location.origin) {
    return `${window.location.origin}/api`;
  }
  return DEFAULT_API_BASE;
}

export async function openReplitWithEnv(opts: {
  apiBase: string;
  x: number;
  y: number;
  energy?: number;
}) {
  const text = envFileContent(opts);
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    /* clipboard may be blocked until user gesture completes */
  }
  window.open(authorLinks.replit, "_blank", "noopener,noreferrer");
  return text;
}
