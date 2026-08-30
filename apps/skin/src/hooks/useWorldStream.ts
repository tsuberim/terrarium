import { useEffect, useRef, useState } from "react";
import type { Creature, CreatureAction, SimConfig, WorldEvent, WorldTile } from "../lib/api";
import { wsRoot } from "../lib/config";
import { WorldRuntime } from "../lib/worldRuntime";

type DeltaMsg = {
  type: "delta";
  tick: number;
  full?: boolean;
  deploy_cost?: number;
  corpse_energy?: number;
  sim_config?: SimConfig;
  creatures_upsert: Creature[];
  creatures_remove: string[];
  tiles_upsert: WorldTile[];
  tiles_remove: [number, number][];
  actions?: CreatureAction[];
  events?: WorldEvent[];
};

export type FxEvent = WorldEvent & {
  at: number;
  simTick?: number;
};

function tileKey(x: number, y: number) {
  return `${x},${y}`;
}

function mergeCreature(prev: Creature | undefined, next: Creature): Creature {
  return {
    ...prev,
    ...next,
    program_hash: next.program_hash ?? prev?.program_hash,
  };
}

export function useWorldStream() {
  const [creatures, setCreatures] = useState<Creature[]>([]);
  const [tiles, setTiles] = useState<WorldTile[]>([]);
  const [deployCost, setDeployCost] = useState(10_000_000);
  const [corpseEnergy, setCorpseEnergy] = useState(1_000_000);
  const [simConfig, setSimConfig] = useState<SimConfig | null>(null);
  const [tick, setTick] = useState(0);
  const [connected, setConnected] = useState(false);
  const [fxEvents, setFxEvents] = useState<FxEvent[]>([]);

  const creaturesMap = useRef(new Map<string, Creature>());
  const tilesMap = useRef(new Map<string, WorldTile>());
  const runtimeRef = useRef(new WorldRuntime());

  const mergeCreatureMeta = useRef((updates: Creature[]) => {
    for (const c of updates) {
      creaturesMap.current.set(c.id, mergeCreature(creaturesMap.current.get(c.id), c));
    }
    setCreatures([...creaturesMap.current.values()]);
  });

  useEffect(() => {
    const prune = window.setInterval(() => {
      const cutoff = Date.now() - 3500;
      setFxEvents((prev) => prev.filter((e) => e.at > cutoff));
    }, 200);
    return () => window.clearInterval(prune);
  }, []);

  useEffect(() => {
    let ws: WebSocket | null = null;
    let retry: number | undefined;
    let closed = false;

    const applyDelta = (msg: DeltaMsg) => {
      if (msg.full) {
        creaturesMap.current = new Map(msg.creatures_upsert.map((c) => [c.id, c]));
        tilesMap.current = new Map(msg.tiles_upsert.map((t) => [tileKey(t.x, t.y), t]));
        runtimeRef.current.reset(msg.tick, msg.creatures_upsert);
        if (msg.deploy_cost !== undefined) setDeployCost(msg.deploy_cost);
        if (msg.corpse_energy !== undefined) setCorpseEnergy(msg.corpse_energy);
        if (msg.sim_config) setSimConfig(msg.sim_config);
        setCreatures([...creaturesMap.current.values()]);
        setTiles([...tilesMap.current.values()]);
        setTick(msg.tick);
        return;
      }

      const fxNow = Date.now();
      const events = (msg.events ?? []).map((e) => ({ ...e, at: fxNow, simTick: msg.tick }) as FxEvent);

      const removedTiles: { x: number; y: number; tile: WorldTile }[] = [];
      for (const [x, y] of msg.tiles_remove) {
        const tile = tilesMap.current.get(tileKey(x, y));
        if (tile) removedTiles.push({ x, y, tile });
      }

      for (const id of msg.creatures_remove) creaturesMap.current.delete(id);
      for (const c of msg.creatures_upsert) {
        creaturesMap.current.set(c.id, mergeCreature(creaturesMap.current.get(c.id), c));
      }
      for (const [x, y] of msg.tiles_remove) tilesMap.current.delete(tileKey(x, y));
      for (const t of msg.tiles_upsert) {
        const prev = tilesMap.current.get(tileKey(t.x, t.y));
        tilesMap.current.set(tileKey(t.x, t.y), { ...prev, ...t, death_reason: t.death_reason ?? prev?.death_reason });
      }

      const upserted = msg.creatures_upsert.map((c) => creaturesMap.current.get(c.id)!);
      runtimeRef.current.upsertCreatures(upserted);
      runtimeRef.current.push(
        {
          tick: msg.tick,
          actions: msg.actions ?? [],
          events,
          removed: msg.creatures_remove,
          removedTiles,
        },
        fxNow,
      );

      setCreatures([...creaturesMap.current.values()]);
      setTiles([...tilesMap.current.values()]);
      setTick(msg.tick);
      if (events.length) {
        setFxEvents((prev) => [...prev, ...events].slice(-96));
      }
    };

    const connect = () => {
      if (closed) return;
      ws = new WebSocket(`${wsRoot()}/v1/world/ws`);

      ws.onopen = () => setConnected(true);

      ws.onmessage = (ev) => {
        try {
          const msg = JSON.parse(String(ev.data)) as DeltaMsg;
          if (msg.type === "delta") applyDelta(msg);
        } catch {
          /* ignore malformed */
        }
      };

      ws.onclose = () => {
        setConnected(false);
        if (!closed) retry = window.setTimeout(connect, 1500);
      };

      ws.onerror = () => ws?.close();
    };

    connect();

    return () => {
      closed = true;
      if (retry) window.clearTimeout(retry);
      const socket = ws;
      if (!socket) return;
      socket.onopen = null;
      socket.onmessage = null;
      socket.onerror = null;
      socket.onclose = null;
      if (socket.readyState === WebSocket.CONNECTING) {
        socket.addEventListener("open", () => socket.close(), { once: true });
      } else {
        socket.close();
      }
    };
  }, []);

  return {
    creatures,
    tiles,
    deployCost,
    corpseEnergy,
    simConfig,
    tick,
    connected,
    fxEvents,
    runtimeRef,
    setSimConfig,
    mergeCreatureMeta: mergeCreatureMeta.current,
  };
}
