import type { CreatureAction, WorldEvent } from "./api";

/** World event enriched with client receive time (for FX scheduling). */
export type FxEvent = WorldEvent & {
  at: number;
  simTick?: number;
};

export type RemovedTile = { x: number; y: number; tile: import("./api").WorldTile };

export type TickFrame = {
  tick: number;
  actions: CreatureAction[];
  events: FxEvent[];
  removed: string[];
  removedTiles: RemovedTile[];
};
