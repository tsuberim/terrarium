import { DEFAULT_ZOOM, type CameraView, type NavFocus, type NavState, parseLocation, writeLocation } from "./navigation";

const KEY = "terrarium.viewer";
const MIN_ZOOM = 0.35;
const MAX_ZOOM = 8;
export const DEFAULT_STUDIO_WIDTH_PCT = 100 / 3;
export const MIN_STUDIO_WIDTH_PCT = 20;
export const MAX_STUDIO_WIDTH_PCT = 75;
export const DEFAULT_STUDIO_CODE_HEIGHT_PCT = 60;
export const MIN_STUDIO_CODE_HEIGHT_PCT = 30;
export const MAX_STUDIO_CODE_HEIGHT_PCT = 75;

export type ViewerPrefs = {
  view: CameraView;
  followId: string | null;
  focus: NavFocus | null;
  zoom: number;
  studioOpen: boolean;
  deployCell: NavFocus | null;
  studioWidthPct: number;
  studioCodeHeightPct: number;
};

const DEFAULT: ViewerPrefs = {
  view: "god",
  followId: null,
  focus: null,
  zoom: DEFAULT_ZOOM,
  studioOpen: false,
  deployCell: null,
  studioWidthPct: DEFAULT_STUDIO_WIDTH_PCT,
  studioCodeHeightPct: DEFAULT_STUDIO_CODE_HEIGHT_PCT,
};

export function clampZoom(z: number) {
  return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, z));
}

export function clampStudioWidthPct(pct: number) {
  return Math.min(MAX_STUDIO_WIDTH_PCT, Math.max(MIN_STUDIO_WIDTH_PCT, pct));
}

export function clampStudioCodeHeightPct(pct: number) {
  return Math.min(MAX_STUDIO_CODE_HEIGHT_PCT, Math.max(MIN_STUDIO_CODE_HEIGHT_PCT, pct));
}

function normalizeFocus(value: unknown): NavFocus | null {
  if (!value || typeof value !== "object") return null;
  const { x, y } = value as { x?: unknown; y?: unknown };
  if (typeof x !== "number" || typeof y !== "number") return null;
  return { x, y };
}

export function loadViewerPrefs(): ViewerPrefs {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return DEFAULT;
    const parsed = JSON.parse(raw) as Partial<ViewerPrefs>;
    return {
      view: parsed.view === "follow" ? "follow" : "god",
      followId: typeof parsed.followId === "string" ? parsed.followId : null,
      focus: normalizeFocus(parsed.focus),
      zoom: typeof parsed.zoom === "number" ? clampZoom(parsed.zoom) : DEFAULT.zoom,
      studioOpen: parsed.studioOpen === true,
      deployCell: normalizeFocus(parsed.deployCell),
      studioWidthPct:
        typeof parsed.studioWidthPct === "number"
          ? clampStudioWidthPct(parsed.studioWidthPct)
          : DEFAULT.studioWidthPct,
      studioCodeHeightPct:
        typeof parsed.studioCodeHeightPct === "number"
          ? clampStudioCodeHeightPct(parsed.studioCodeHeightPct)
          : DEFAULT.studioCodeHeightPct,
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
  return (
    params.has("creature") ||
    (params.has("x") && params.has("y")) ||
    params.has("z") ||
    params.has("studio")
  );
}

function urlLocationState(): NavState {
  const parsed = parseLocation();
  return { ...parsed, zoom: clampZoom(parsed.zoom) };
}

export function resolveInitialViewerState(): ViewerPrefs {
  const stored = loadViewerPrefs();
  if (!hasLocationParams()) return stored;

  const url = urlLocationState();
  const deployCell =
    url.studioOpen && url.focus ? url.focus : stored.deployCell;

  return {
    view: url.view,
    followId: url.followId,
    focus: url.studioOpen ? stored.focus : url.focus,
    zoom: url.zoom,
    studioOpen: url.studioOpen || stored.studioOpen,
    deployCell,
    studioWidthPct: stored.studioWidthPct,
    studioCodeHeightPct: stored.studioCodeHeightPct,
  };
}

function urlFocus(prefs: ViewerPrefs): NavFocus | null {
  if (prefs.view === "follow") return null;
  if (prefs.studioOpen && prefs.deployCell) return prefs.deployCell;
  return prefs.focus;
}

export function persistViewerState(prefs: ViewerPrefs) {
  const zoom = clampZoom(prefs.zoom);
  const next: ViewerPrefs = { ...prefs, zoom };
  writeLocation({
    view: next.view,
    followId: next.followId,
    focus: urlFocus(next),
    zoom,
    studioOpen: next.studioOpen,
  });
  saveViewerPrefs(next);
}
