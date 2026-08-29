import { useEffect, useRef, useState } from "react";
import type { Creature, SimConfig, WorldEvent, WorldTile } from "../lib/api";
import { wsRoot } from "../lib/config";

type SnapshotMsg = {
  type: "snapshot";
  tick: number;
  deploy_cost: number;
  corpse_energy: number;
  sim_config?: SimConfig;
  creatures: Creature[];
  tiles: WorldTile[];
};

type DeltaMsg = {
  type: "delta";
  tick: number;
  creatures_upsert: Creature[];
  creatures_remove: string[];
  tiles_upsert: WorldTile[];
  tiles_remove: [number, number][];
  events?: WorldEvent[];
};

type WorldMsg = SnapshotMsg | DeltaMsg;

export type FxEvent = WorldEvent & { at: number };

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

  const mergeCreatureMeta = useRef((updates: Creature[]) => {
    for (const c of updates) {
      creaturesMap.current.set(c.id, mergeCreature(creaturesMap.current.get(c.id), c));
    }
    setCreatures([...creaturesMap.current.values()]);
  });

  useEffect(() => {
    const prune = window.setInterval(() => {
      const cutoff = Date.now() - 2500;
      setFxEvents((prev) => prev.filter((e) => e.at > cutoff));
    }, 200);
    return () => window.clearInterval(prune);
  }, []);

  useEffect(() => {
    let ws: WebSocket | null = null;
    let retry: number | undefined;
    let closed = false;

    const applySnapshot = (msg: SnapshotMsg) => {
      creaturesMap.current = new Map(msg.creatures.map((c) => [c.id, c]));
      tilesMap.current = new Map(msg.tiles.map((t) => [tileKey(t.x, t.y), t]));
      setCreatures([...creaturesMap.current.values()]);
      setTiles([...tilesMap.current.values()]);
      setDeployCost(msg.deploy_cost);
      setCorpseEnergy(msg.corpse_energy);
      if (msg.sim_config) setSimConfig(msg.sim_config);
      setTick(msg.tick);
    };

    const applyDelta = (msg: DeltaMsg) => {
      for (const id of msg.creatures_remove) {
        creaturesMap.current.delete(id);
      }
      for (const c of msg.creatures_upsert) {
        creaturesMap.current.set(c.id, mergeCreature(creaturesMap.current.get(c.id), c));
      }
      for (const [x, y] of msg.tiles_remove) {
        tilesMap.current.delete(tileKey(x, y));
      }
      for (const t of msg.tiles_upsert) {
        const prev = tilesMap.current.get(tileKey(t.x, t.y));
        tilesMap.current.set(tileKey(t.x, t.y), { ...prev, ...t, death_reason: t.death_reason ?? prev?.death_reason });
      }
      setCreatures([...creaturesMap.current.values()]);
      setTiles([...tilesMap.current.values()]);
      setTick(msg.tick);
      if (msg.events?.length) {
        const now = Date.now();
        setFxEvents((prev) => [...prev, ...msg.events!.map((e) => ({ ...e, at: now }))].slice(-96));
      }
    };

    const connect = () => {
      if (closed) return;
      ws = new WebSocket(`${wsRoot()}/v1/world/ws`);

      ws.onopen = () => setConnected(true);

      ws.onmessage = (ev) => {
        try {
          const msg = JSON.parse(String(ev.data)) as WorldMsg;
          if (msg.type === "snapshot") applySnapshot(msg);
          else if (msg.type === "delta") applyDelta(msg);
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
    setSimConfig,
    mergeCreatureMeta: mergeCreatureMeta.current,
  };
}
