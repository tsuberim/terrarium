/**
 * Client world runtime: explicit tick actions → animation → render poses.
 *
 * WS delivers `{ tick, actions, events, ... }`. Actions drive motion; upserts
 * carry stats. No diff inference.
 */

import type { Creature, CreatureAction, WorldTile } from "./api";
import type { FxEvent, TickFrame } from "./worldTypes";
import { mouthEnvelope, EAT_LIFE_MS } from "./eatFx";
import { hitFireIntensity, HIT_LIFE_MS } from "./hitFx";
import { SPAWN_LIFE_MS, DEATH_LIFE_MS } from "./creatureSprite";
import { facingAngle, lerpAngle, axialToPixel } from "./hex";

export type { TickFrame, FxEvent } from "./worldTypes";

export type Pose = {
  q: number;
  r: number;
  px: number;
  py: number;
  angle: number;
  moving: boolean;
  rotating: boolean;
};

export type RenderFrame = {
  simTick: number;
  poses: ReadonlyMap<string, Pose>;
  spawnLife: ReadonlyMap<string, { at: number; fromQ: number; fromR: number }>;
  deathGhosts: ReadonlyMap<string, { at: number; creature: Creature }>;
};

type Anchor = { q: number; r: number; facing: number };

type Segment =
  | { kind: "move"; fromQ: number; fromR: number; toQ: number; toR: number; start: number; end: number }
  | { kind: "rotate"; from: number; to: number; toFacing: number; start: number; end: number };

function easeOut(t: number) {
  return 1 - (1 - t) ** 3;
}

function fxKey(fx: FxEvent): string {
  if (fx.type === "spawn") return `spawn:${fx.creature_id}:${fx.at}`;
  if (fx.type === "death") return `death:${fx.creature_id}:${fx.at}`;
  if (fx.type === "eat") return `eat:${fx.actor_id}:${fx.x},${fx.y}:${fx.at}`;
  if (fx.type === "hit") return `hit:${fx.victim_id}:${fx.at}`;
  return `${fx.type}:${fx.at}`;
}

function tileKey(x: number, y: number) {
  return `${x},${y}`;
}

function normFacing(f: number) {
  return ((f % 6) + 6) % 6;
}

function deathCreature(fx: FxEvent & { type: "death" }): Creature {
  return {
    id: fx.creature_id,
    x: fx.x,
    y: fx.y,
    owner_uid: fx.owner_uid,
    facing: fx.facing,
    energy: fx.energy,
    health: fx.health,
    max_health: fx.max_health,
  };
}

function poseFromAnchor(a: Anchor): Pose {
  const { x: px, y: py } = axialToPixel(a.q, a.r);
  return { q: a.q, r: a.r, px, py, angle: facingAngle(a.facing), moving: false, rotating: false };
}

function resolvePose(anchor: Anchor, segments: Segment[], perfNow: number): Pose {
  let q = anchor.q;
  let r = anchor.r;
  let angle = facingAngle(anchor.facing);

  for (const seg of segments) {
    if (perfNow >= seg.end) {
      if (seg.kind === "move") {
        q = seg.toQ;
        r = seg.toR;
      } else {
        angle = seg.to;
      }
      continue;
    }
    if (perfNow >= seg.start) {
      const t = easeOut((perfNow - seg.start) / (seg.end - seg.start));
      if (seg.kind === "move") {
        const from = axialToPixel(seg.fromQ, seg.fromR);
        const to = axialToPixel(seg.toQ, seg.toR);
        return {
          q: seg.fromQ + (seg.toQ - seg.fromQ) * t,
          r: seg.fromR + (seg.toR - seg.fromR) * t,
          px: from.x + (to.x - from.x) * t,
          py: from.y + (to.y - from.y) * t,
          angle,
          moving: seg.fromQ !== seg.toQ || seg.fromR !== seg.toR,
          rotating: false,
        };
      }
      return {
        q,
        r,
        px: axialToPixel(q, r).x,
        py: axialToPixel(q, r).y,
        angle: lerpAngle(seg.from, seg.to, t),
        moving: false,
        rotating: Math.abs(seg.to - seg.from) > 0.001,
      };
    }
    break;
  }

  return { q, r, px: axialToPixel(q, r).x, py: axialToPixel(q, r).y, angle, moving: false, rotating: false };
}

function commitAnchor(anchor: Anchor, segments: Segment[], perfNow: number): Anchor {
  let { q, r, facing } = anchor;
  for (const seg of segments) {
    if (perfNow < seg.end) break;
    if (seg.kind === "move") {
      q = seg.toQ;
      r = seg.toR;
    } else {
      facing = seg.toFacing;
    }
  }
  return { q, r, facing };
}

export class WorldRuntime {
  private tickMs = 500;
  private simTick = 0;

  private anchors = new Map<string, Anchor>();
  private segments = new Map<string, Segment[]>();
  private queueEnd = new Map<string, number>();
  private creatures = new Map<string, Creature>();

  private fxLog: FxEvent[] = [];
  private fxSeen = new Set<string>();
  private spawnLife = new Map<string, { at: number; fromQ: number; fromR: number }>();
  private deathGhosts = new Map<string, { at: number; creature: Creature }>();
  private hitAt = new Map<string, number>();
  private eatStart = new Map<string, number>();
  private heldTiles = new Map<string, { tile: WorldTile; hideAfter: number }>();

  private poses = new Map<string, Pose>();

  setTickHz(hz: number) {
    this.tickMs = 1000 / hz;
  }

  reset(tick: number, creatures: Creature[]) {
    const now = performance.now();
    this.simTick = tick;
    this.anchors.clear();
    this.segments.clear();
    this.queueEnd.clear();
    this.creatures.clear();
    this.fxLog = [];
    this.fxSeen.clear();
    this.spawnLife.clear();
    this.deathGhosts.clear();
    this.hitAt.clear();
    this.eatStart.clear();
    this.heldTiles.clear();
    this.poses.clear();
    for (const c of creatures) {
      this.creatures.set(c.id, c);
      this.anchors.set(c.id, { q: c.x, r: c.y, facing: normFacing(c.facing ?? 0) });
      this.queueEnd.set(c.id, now);
    }
  }

  /** Apply creature stat upserts (not motion). */
  upsertCreatures(creatures: Creature[]) {
    for (const c of creatures) {
      const prev = this.creatures.get(c.id);
      this.creatures.set(c.id, prev ? { ...prev, ...c } : c);
      if (!this.anchors.has(c.id)) {
        this.anchors.set(c.id, { q: c.x, r: c.y, facing: normFacing(c.facing ?? 0) });
        this.queueEnd.set(c.id, performance.now());
      }
    }
  }

  removeCreatures(ids: string[]) {
    for (const id of ids) {
      this.creatures.delete(id);
      this.anchors.delete(id);
      this.segments.delete(id);
      this.queueEnd.delete(id);
      this.poses.delete(id);
    }
  }

  push(frame: TickFrame, fxNow: number) {
    const perfNow = performance.now();
    this.simTick = Math.max(this.simTick, frame.tick);

    for (const action of frame.actions) {
      this.applyAction(action, perfNow);
    }

    for (const fx of frame.events) {
      this.registerFx(fx, fxNow, perfNow);
      this.fxLog.push(fx);
    }

    for (const { x, y, tile } of frame.removedTiles) {
      const eatFx = frame.events.find(
        (e): e is FxEvent & { type: "eat" } => e.type === "eat" && e.x === x && e.y === y,
      );
      if (!eatFx) continue;
      const start = this.eatStartAt(eatFx);
      this.heldTiles.set(tileKey(x, y), { tile, hideAfter: start + EAT_LIFE_MS });
    }

    for (const id of frame.removed) {
      this.removeCreatures([id]);
    }

    if (this.fxLog.length > 128) this.fxLog.splice(0, this.fxLog.length - 96);
  }

  private applyAction(action: CreatureAction, perfNow: number) {
    const id = action.creature_id;
    if (!this.anchors.has(id)) {
      const c = this.creatures.get(id);
      this.anchors.set(
        id,
        c
          ? { q: c.x, r: c.y, facing: normFacing(c.facing ?? 0) }
          : { q: 0, r: 0, facing: 0 },
      );
      this.queueEnd.set(id, perfNow);
    }

    const segs = this.segments.get(id) ?? [];
    let at = Math.max(perfNow, this.queueEnd.get(id) ?? perfNow);

    switch (action.kind) {
      case "rotate": {
        segs.push({
          kind: "rotate",
          from: facingAngle(action.from_facing),
          to: facingAngle(action.to_facing),
          toFacing: normFacing(action.to_facing),
          start: at,
          end: at + this.tickMs,
        });
        at += this.tickMs;
        break;
      }
      case "move": {
        segs.push({
          kind: "move",
          fromQ: action.from_x,
          fromR: action.from_y,
          toQ: action.to_x,
          toR: action.to_y,
          start: at,
          end: at + this.tickMs,
        });
        at += this.tickMs;
        break;
      }
      case "eat": {
        at = Math.max(at, perfNow);
        break;
      }
      case "hit": {
        at += this.tickMs * 0.5;
        break;
      }
    }

    if (action.kind === "rotate" || action.kind === "move") {
      this.segments.set(id, segs.filter((s) => s.end > perfNow - this.tickMs * 3));
      this.queueEnd.set(id, at);
    }
  }

  private scheduleEat(actorId: string, start: number) {
    this.queueEnd.set(actorId, Math.max(this.queueEnd.get(actorId) ?? 0, start + EAT_LIFE_MS));
  }

  private registerFx(fx: FxEvent, fxNow: number, perfNow: number) {
    const key = fxKey(fx);
    if (this.fxSeen.has(key)) return;
    this.fxSeen.add(key);

    if (fx.type === "spawn") {
      this.spawnLife.set(fx.creature_id, {
        at: fx.at,
        fromQ: fx.parent_x,
        fromR: fx.parent_y,
      });
    } else if (fx.type === "death") {
      this.deathGhosts.set(fx.creature_id, { at: fx.at, creature: deathCreature(fx) });
    } else if (fx.type === "hit") {
      this.hitAt.set(fx.actor_id, fx.at);
    } else if (fx.type === "eat") {
      const end = this.queueEnd.get(fx.actor_id) ?? perfNow;
      let start = Math.max(end, perfNow);
      if (fx.tile_kind === 3) {
        for (const other of this.fxLog) {
          if (other.type === "death" && other.x === fx.x && other.y === fx.y) {
            const deathEnd = other.at + DEATH_LIFE_MS;
            if (fxNow < deathEnd) start = Math.max(start, perfNow + (deathEnd - fxNow));
          }
        }
        for (const [, ghost] of this.deathGhosts) {
          if (ghost.creature.x === fx.x && ghost.creature.y === fx.y) {
            const deathEnd = ghost.at + DEATH_LIFE_MS;
            if (fxNow < deathEnd) start = Math.max(start, perfNow + (deathEnd - fxNow));
          }
        }
      }
      this.eatStart.set(key, start);
      this.scheduleEat(fx.actor_id, start);
    }
  }

  sample(perfNow: number, fxNow: number): RenderFrame {
    this.poses.clear();
    for (const [id, anchor] of this.anchors) {
      const segs = this.segments.get(id) ?? [];
      const committed = commitAnchor(anchor, segs, perfNow);
      this.anchors.set(id, committed);
      this.poses.set(id, resolvePose(committed, segs, perfNow));
    }

    for (const [id, t] of this.spawnLife) {
      if (fxNow - t.at > SPAWN_LIFE_MS) this.spawnLife.delete(id);
    }
    for (const [id, t] of this.deathGhosts) {
      if (fxNow - t.at > DEATH_LIFE_MS) this.deathGhosts.delete(id);
    }
    for (const [k, t] of this.eatStart) {
      if (perfNow - t > EAT_LIFE_MS) this.eatStart.delete(k);
    }
    for (const [k, held] of this.heldTiles) {
      if (perfNow >= held.hideAfter) this.heldTiles.delete(k);
    }
    for (const [id, t] of this.hitAt) {
      if (fxNow - t > HIT_LIFE_MS) this.hitAt.delete(id);
    }
    if (this.fxSeen.size > 240) this.fxSeen.clear();

    return {
      simTick: this.simTick,
      poses: this.poses,
      spawnLife: this.spawnLife,
      deathGhosts: this.deathGhosts,
    };
  }

  pose(id: string, fallback: Creature): Pose {
    return this.poses.get(id) ?? poseFromAnchor({ q: fallback.x, r: fallback.y, facing: normFacing(fallback.facing ?? 0) });
  }

  busy(id: string): boolean {
    const p = this.poses.get(id);
    return !!p && (p.moving || p.rotating);
  }

  eatOpen(id: string, perfNow: number): number {
    if (this.busy(id)) return 0;
    let open = 0;
    for (const fx of this.fxLog) {
      if (fx.type !== "eat" || fx.actor_id !== id) continue;
      const start = this.eatStart.get(fxKey(fx)) ?? fx.at;
      if (perfNow < start) continue;
      const age = (perfNow - start) / EAT_LIFE_MS;
      if (age >= 0 && age < 1) open = Math.max(open, mouthEnvelope(age));
    }
    return open;
  }

  eatStartAt(fx: FxEvent & { type: "eat" }): number {
    return this.eatStart.get(fxKey(fx)) ?? fx.at;
  }

  hitFire(id: string, fxNow: number): number {
    const t = this.hitAt.get(id);
    return t !== undefined ? hitFireIntensity(t, fxNow) : 0;
  }

  /** FX log for canvas (single source; React EventFeed uses separate HUD copy). */
  fxForRender(): readonly FxEvent[] {
    return this.fxLog;
  }

  /** Sim tiles merged with held tiles; hides cells mid-eat/death FX. */
  displayTiles(simTiles: WorldTile[], perfNow: number, fxNow: number): WorldTile[] {
    const map = new Map(simTiles.map((t) => [tileKey(t.x, t.y), t]));
    for (const [key, held] of this.heldTiles) {
      if (perfNow < held.hideAfter) map.set(key, held.tile);
    }
    const out: WorldTile[] = [];
    for (const t of map.values()) {
      if (this.tileHiddenDuringFx(t.x, t.y, perfNow, fxNow)) continue;
      out.push(t);
    }
    return out;
  }

  private tileHiddenDuringFx(x: number, y: number, perfNow: number, fxNow: number): boolean {
    for (const [, g] of this.deathGhosts) {
      if (g.creature.x === x && g.creature.y === y && fxNow - g.at < DEATH_LIFE_MS) return true;
    }
    for (const fx of this.fxLog) {
      if (fx.type !== "eat" || fx.x !== x || fx.y !== y) continue;
      const start = this.eatStartAt(fx);
      if (perfNow < start) continue;
      const age = (perfNow - start) / EAT_LIFE_MS;
      if (age >= 0 && age < 1) return true;
    }
    return false;
  }

  corpseHidden(x: number, y: number, perfNow: number, fxNow: number): boolean {
    return this.tileHiddenDuringFx(x, y, perfNow, fxNow);
  }
}
