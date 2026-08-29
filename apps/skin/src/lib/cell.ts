import type { Creature, WorldTile } from "./api";
import { formatDeathReason } from "./death";

export function describeCell(
  x: number,
  y: number,
  tiles: WorldTile[],
  creatures: Creature[],
  userUid?: string,
): string {
  const creature = creatures.find((c) => c.x === x && c.y === y);
  if (creature) {
    const who = creature.owner_uid === userUid ? "Your creature" : "Creature";
    return `${who} · ${creature.energy} energy`;
  }

  const tile = tiles.find((t) => t.x === x && t.y === y);
  if (!tile) return "Empty ground";
  if (tile.kind === 1) return "Solid wall";
  if (tile.kind === 3) {
    const reason = tile.death_reason ? formatDeathReason(tile.death_reason) : null;
    return reason
      ? `Corpse · ${tile.energy ?? 0} energy · ${reason}`
      : `Corpse · ${tile.energy ?? 0} energy`;
  }
  return "Unknown";
}
