import { useCallback, useEffect, useRef, useState } from "react";
import type { Creature, WorldTile } from "../lib/api";
import type { SandboxResult } from "../lib/creatureEditor";
import { WorldRuntime } from "../lib/worldRuntime";
import type { FxEvent } from "../lib/worldTypes";

const SANDBOX_ID = "sandbox";

export type PlaybackState = "stopped" | "playing" | "paused";

export function useSandboxPlayback(tickHz: number) {
  const [result, setResult] = useState<SandboxResult | null>(null);
  const [frameIndex, setFrameIndex] = useState(0);
  const [playback, setPlayback] = useState<PlaybackState>("stopped");

  const creaturesLiveRef = useRef<Creature[]>([]);
  const tilesLiveRef = useRef<WorldTile[]>([]);
  const runtimeRef = useRef(new WorldRuntime());

  const applyFrame = useCallback((res: SandboxResult, index: number) => {
    const frame = res.frames[Math.min(index, res.frames.length - 1)];
    if (!frame) return;

    const maxHealth = frame.health > 0 ? Math.max(frame.health, 100) : 100;
    creaturesLiveRef.current = [
      {
        id: SANDBOX_ID,
        x: frame.x,
        y: frame.y,
        energy: frame.energy,
        health: frame.health,
        max_health: maxHealth,
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
    for (let i = 0; i <= index; i++) {
      const f = res.frames[i]!;
      rt.push(
        {
          tick: f.tick,
          actions: f.actions ?? [],
          events: (f.events ?? []).map((e) => ({ ...e, at: fxNow, simTick: f.tick }) as FxEvent),
          removed: [],
          removedTiles: [],
        },
        fxNow,
      );
    }
  }, [tickHz]);

  const loadResult = useCallback(
    (res: SandboxResult | null) => {
      setResult(res);
      setFrameIndex(0);
      setPlayback("stopped");
      if (!res || !res.frames.length) {
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
      if (!result) return;
      const next = Math.max(0, Math.min(index, result.frames.length - 1));
      setFrameIndex(next);
      applyFrame(result, next);
    },
    [applyFrame, result],
  );

  const play = useCallback(() => {
    if (!result?.frames.length) return;
    setFrameIndex((prev) => {
      if (result && prev >= result.frames.length - 1) {
        applyFrame(result, 0);
        return 0;
      }
      return prev;
    });
    setPlayback("playing");
  }, [applyFrame, result]);

  const pause = useCallback(() => setPlayback("paused"), []);

  const stop = useCallback(() => {
    setPlayback("stopped");
    seek(0);
  }, [seek]);

  useEffect(() => {
    if (playback !== "playing" || !result?.frames.length) return;
    const ms = 1000 / tickHz;
    const id = window.setInterval(() => {
      setFrameIndex((prev) => {
        const next = prev + 1;
        if (next >= result.frames.length) {
          setPlayback("paused");
          return prev;
        }
        applyFrame(result, next);
        return next;
      });
    }, ms);
    return () => window.clearInterval(id);
  }, [playback, result, tickHz, applyFrame]);

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
