import type { Creature } from "./api";

export type CameraView = "god" | "follow";

export type NavFocus = { x: number; y: number };

export type NavState = {
  view: CameraView;
  followId: string | null;
  focus: NavFocus | null;
  zoom: number;
  studioOpen: boolean;
};

export const DEFAULT_ZOOM = 1;

export function parseLocation(search = window.location.search): NavState {
  const params = new URLSearchParams(search);
  const studioOpen = params.get("studio") === "1";
  const zoomRaw = params.get("z");
  const zoom =
    zoomRaw !== null && Number.isFinite(Number.parseFloat(zoomRaw))
      ? Number.parseFloat(zoomRaw)
      : DEFAULT_ZOOM;

  const creature = params.get("creature")?.trim();
  if (creature) {
    return { view: "follow", followId: creature, focus: null, zoom, studioOpen };
  }

  const xRaw = params.get("x");
  const yRaw = params.get("y");
  if (xRaw !== null && yRaw !== null) {
    const x = Number.parseInt(xRaw, 10);
    const y = Number.parseInt(yRaw, 10);
    if (Number.isFinite(x) && Number.isFinite(y)) {
      return { view: "god", followId: null, focus: { x, y }, zoom, studioOpen };
    }
  }

  return { view: "god", followId: null, focus: null, zoom, studioOpen };
}

export function writeLocation(state: NavState) {
  const params = new URLSearchParams();

  if (state.studioOpen) {
    params.set("studio", "1");
  }

  if (state.view === "follow" && state.followId) {
    params.set("creature", state.followId);
  } else if (state.focus) {
    params.set("x", String(state.focus.x));
    params.set("y", String(state.focus.y));
  }

  if (Math.abs(state.zoom - DEFAULT_ZOOM) > 0.001) {
    params.set("z", state.zoom.toFixed(3).replace(/\.?0+$/, ""));
  }

  const qs = params.toString();
  const next = qs ? `${window.location.pathname}?${qs}` : window.location.pathname;
  const current = `${window.location.pathname}${window.location.search}`;
  if (next !== current) {
    window.history.replaceState(null, "", next);
  }
}

export function parseJumpQuery(query: string): { type: "coords"; x: number; y: number } | { type: "creature"; id: string } | null {
  const q = query.trim();
  if (!q) return null;

  const coordMatch = q.match(/^\(?\s*(-?\d+)\s*[, ]\s*(-?\d+)\s*\)?$/);
  if (coordMatch) {
    return { type: "coords", x: Number.parseInt(coordMatch[1], 10), y: Number.parseInt(coordMatch[2], 10) };
  }

  if (/^[0-9a-f-]+$/i.test(q)) {
    return { type: "creature", id: q };
  }

  return null;
}

export function resolveCreatureId(query: string, creatures: Creature[]): string | null {
  const q = query.trim().toLowerCase();
  if (!q) return null;

  const exact = creatures.find((c) => c.id.toLowerCase() === q);
  if (exact) return exact.id;

  const prefixMatches = creatures.filter((c) => c.id.toLowerCase().startsWith(q));
  if (prefixMatches.length === 1) return prefixMatches[0].id;

  return null;
}

export function creatureMatches(query: string, creatures: Creature[]): Creature[] {
  const q = query.trim().toLowerCase();
  if (!q) return creatures.slice(0, 8);

  const parsed = parseJumpQuery(q);
  if (parsed?.type === "creature") {
    return creatures
      .filter((c) => c.id.toLowerCase().startsWith(parsed.id.toLowerCase()))
      .slice(0, 8);
  }

  return creatures
    .filter((c) => c.id.toLowerCase().includes(q))
    .slice(0, 8);
}
