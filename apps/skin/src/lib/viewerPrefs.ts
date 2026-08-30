import type { CameraView, NavFocus } from "./navigation";

const KEY = "terrarium.viewer";
const MIN_ZOOM = 0.35;
const MAX_ZOOM = 8;

export type ViewerPrefs = {
  view: CameraView;
  followId: string | null;
  focus: NavFocus | null;
  zoom: number;
  spriteMode: "id" | "hash";
};

const DEFAULT: ViewerPrefs = {
  view: "god",
  followId: null,
  focus: null,
  zoom: 1,
  spriteMode: "id",
};

function clampZoom(z: number) {
  return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, z));
}

export function loadViewerPrefs(): ViewerPrefs {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return DEFAULT;
    const parsed = JSON.parse(raw) as Partial<ViewerPrefs>;
    return {
      view: parsed.view === "follow" ? "follow" : "god",
      followId: typeof parsed.followId === "string" ? parsed.followId : null,
      focus:
        parsed.focus &&
        typeof parsed.focus.x === "number" &&
        typeof parsed.focus.y === "number"
          ? { x: parsed.focus.x, y: parsed.focus.y }
          : null,
      zoom: typeof parsed.zoom === "number" ? clampZoom(parsed.zoom) : DEFAULT.zoom,
      spriteMode: parsed.spriteMode === "hash" ? "hash" : "id",
    };
  } catch {
    return DEFAULT;
  }
}

export function saveViewerPrefs(prefs: ViewerPrefs) {
  try {
    localStorage.setItem(KEY, JSON.stringify(prefs));
  } catch {
    /* private mode / quota */
  }
}

export function hasLocationParams(search = window.location.search): boolean {
  const params = new URLSearchParams(search);
  return params.has("creature") || (params.has("x") && params.has("y"));
}

export function resolveInitialViewerState(): ViewerPrefs {
  if (hasLocationParams()) {
    const params = new URLSearchParams(window.location.search);
    const creature = params.get("creature")?.trim();
    if (creature) {
      return { ...loadViewerPrefs(), view: "follow", followId: creature, focus: null };
    }
    const x = Number.parseInt(params.get("x") ?? "", 10);
    const y = Number.parseInt(params.get("y") ?? "", 10);
    if (Number.isFinite(x) && Number.isFinite(y)) {
      return { ...loadViewerPrefs(), view: "god", followId: null, focus: { x, y } };
    }
  }
  return loadViewerPrefs();
}
