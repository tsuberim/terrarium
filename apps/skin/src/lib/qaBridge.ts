export type QaPlayback = "idle" | "playing" | "paused";

export type StudioQaSlice = {
  testing: boolean;
  wasmReady: boolean;
  playback: QaPlayback;
  error: string | null;
};

export type QaState = {
  ready: boolean;
  signedIn: boolean;
  studioOpen: boolean;
  deployCell: { x: number; y: number } | null;
  deployDialogOpen: boolean;
  credits: number | null;
  testing: boolean;
  wasmReady: boolean;
  playback: QaPlayback;
  error: string | null;
  busy: boolean;
};

export type QaBridge = {
  getState: () => QaState;
  waitFor: (predicate: (state: QaState) => boolean, timeoutMs?: number) => Promise<QaState>;
};

declare global {
  interface Window {
    __TERRARIUM_QA__?: QaBridge;
  }
}

export function mapPlayback(playback: "stopped" | "playing" | "paused"): QaPlayback {
  if (playback === "playing") return "playing";
  if (playback === "paused") return "paused";
  return "idle";
}
