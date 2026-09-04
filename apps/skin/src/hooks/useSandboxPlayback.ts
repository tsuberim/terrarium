import { useCallback, useEffect, useRef, useState } from "react";
import type { Creature, WorldTile } from "../lib/api";
import type { SandboxResult } from "../lib/creatureEditor";
import { WorldRuntime } from "../lib/worldRuntime";
import type { FxEvent } from "../lib/worldTypes";

const SANDBOX_ID = "sandbox";
const SANDBOX_MAX_HEALTH = 100;

export type PlaybackState = "stopped" | "playing" | "paused";

export function useSandboxPlayback(tickHz: number) {
  const [result, setResult] = useState<SandboxResult | null>(null);
  const [frameIndex, setFrameIndex] = useState(0);
  const [playback, setPlayback] = useState<PlaybackState>("stopped");
  const resultRef = useRef<SandboxResult | null>(null);

  const creaturesLiveRef = useRef<Creature[]>([]);
  const tilesLiveRef = useRef<WorldTile[]>([]);
  const runtimeRef = useRef(new WorldRuntime());

  const applyFrame = useCallback((res: SandboxResult, index: number) => {
    const frame = res.frames[Math.min(index, res.frames.length - 1)];
    if (!frame) return;

    creaturesLiveRef.current = [
      {
        id: SANDBOX_ID,
        x: frame.x,
        y: frame.y,
        energy: frame.energy,
        health: frame.health,
        max_health: SANDBOX_MAX_HEALTH,
        owner_uid: "sandbox",
        facing: frame.facing,
      },
    ];

    tilesLiveRef.current = res.tiles.map((t) => ({
      x: t.x,
      y: t.y,
      kind: t.kind,
      energy: t.energy,
    }));

    const rt = runtimeRef.current;
    rt.reset(frame.tick, creaturesLiveRef.current);
    rt.setTickHz(tickHz);
    const fxNow = performance.now();
    rt.push(
      {
        tick: frame.tick,
        actions: frame.actions ?? [],
        events: (frame.events ?? []).map((e) => ({ ...e, at: fxNow, simTick: frame.tick }) as FxEvent),
        removed: [],
        removedTiles: [],
      },
      fxNow,
    );
  }, [tickHz]);

  const loadResult = useCallback(
    (res: SandboxResult | null) => {
      resultRef.current = res;
      setResult(res);
      setFrameIndex(0);
      setPlayback("stopped");
      if (!res?.frames.length) {
        creaturesLiveRef.current = [];
        tilesLiveRef.current = [];
        runtimeRef.current.reset(0, []);
        return;
      }
      applyFrame(res, 0);
    },
    [applyFrame],
  );

  const seek = useCallback(
    (index: number) => {
      const res = resultRef.current;
      if (!res) return;
      const next = Math.max(0, Math.min(index, res.frames.length - 1));
      setFrameIndex(next);
      applyFrame(res, next);
    },
    [applyFrame],
  );

  const play = useCallback(() => {
    const res = resultRef.current;
    if (!res?.frames.length) return;
    setFrameIndex((prev) => {
      const last = res.frames.length - 1;
      const next = prev >= last ? 0 : prev;
      applyFrame(res, next);
      return next;
    });
    setPlayback("playing");
  }, [applyFrame]);

  const pause = useCallback(() => setPlayback("paused"), []);

  const stop = useCallback(() => {
    setPlayback("stopped");
    seek(0);
  }, [seek]);

  useEffect(() => {
    if (playback !== "playing") return;
    const ms = Math.max(50, 1000 / tickHz);
    const id = window.setInterval(() => {
      const res = resultRef.current;
      if (!res?.frames.length) {
        setPlayback("paused");
        return;
      }
      setFrameIndex((prev) => {
        if (prev >= res.frames.length - 1) {
          setPlayback("paused");
          return prev;
        }
        const next = prev + 1;
        applyFrame(res, next);
        return next;
      });
    }, ms);
    return () => window.clearInterval(id);
  }, [playback, tickHz, applyFrame]);

  return {
    result,
    frameIndex,
    playback,
    creaturesLiveRef,
    tilesLiveRef,
    runtimeRef,
    loadResult,
    seek,
    play,
    pause,
    stop,
  };
}
