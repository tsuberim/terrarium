import { useEffect, useRef, useState } from "react";
import type { Creature, WorldTile } from "../lib/api";
import { wsRoot } from "../lib/config";

type SnapshotMsg = {
  type: "snapshot";
  tick: number;
  deploy_cost: number;
  corpse_energy: number;
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
};

type WorldMsg = SnapshotMsg | DeltaMsg;

function tileKey(x: number, y: number) {
  return `${x},${y}`;
}

export function useWorldStream() {
  const [creatures, setCreatures] = useState<Creature[]>([]);
  const [tiles, setTiles] = useState<WorldTile[]>([]);
  const [deployCost, setDeployCost] = useState(100);
  const [corpseEnergy, setCorpseEnergy] = useState(10);
  const [tick, setTick] = useState(0);
  const [connected, setConnected] = useState(false);

  const creaturesMap = useRef(new Map<string, Creature>());
  const tilesMap = useRef(new Map<string, WorldTile>());

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
      setTick(msg.tick);
    };

    const applyDelta = (msg: DeltaMsg) => {
      for (const id of msg.creatures_remove) {
        creaturesMap.current.delete(id);
      }
      for (const c of msg.creatures_upsert) {
        creaturesMap.current.set(c.id, c);
      }
      for (const [x, y] of msg.tiles_remove) {
        tilesMap.current.delete(tileKey(x, y));
      }
      for (const t of msg.tiles_upsert) {
        tilesMap.current.set(tileKey(t.x, t.y), t);
      }
      setCreatures([...creaturesMap.current.values()]);
      setTiles([...tilesMap.current.values()]);
      setTick(msg.tick);
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

  return { creatures, tiles, deployCost, corpseEnergy, tick, connected };
}
