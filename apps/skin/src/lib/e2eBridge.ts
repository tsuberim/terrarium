export type E2ePlayback = "idle" | "playing" | "paused";

export type StudioE2eSlice = {
  testing: boolean;
  wasmReady: boolean;
  allTestsPassed: boolean;
  playback: E2ePlayback;
  error: string | null;
};

export type E2eState = {
  ready: boolean;
  signedIn: boolean;
  studioOpen: boolean;
  deployCell: { x: number; y: number } | null;
  deployDialogOpen: boolean;
  credits: number | null;
  testing: boolean;
  wasmReady: boolean;
  allTestsPassed: boolean;
  playback: E2ePlayback;
  error: string | null;
  busy: boolean;
};

export type E2eBridge = {
  getState: () => E2eState;
  waitFor: (predicate: (state: E2eState) => boolean, timeoutMs?: number) => Promise<E2eState>;
};

declare global {
  interface Window {
    __TERRARIUM_E2E__?: E2eBridge;
  }
}

export function mapPlayback(playback: "stopped" | "playing" | "paused"): E2ePlayback {
  if (playback === "playing") return "playing";
  if (playback === "paused") return "paused";
  return "idle";
}
