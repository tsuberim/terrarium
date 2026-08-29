import type { Creature, WorldTile } from "./api";
import { formatDeathReason } from "./death";
import { formatGlimLabel } from "./glim";

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
    return `${who} · ${creature.health}/${creature.max_health} hp · ${formatGlimLabel(creature.energy)}`;
  }

  const tile = tiles.find((t) => t.x === x && t.y === y);
  if (!tile) return "Empty ground";
  if (tile.kind === 1) return "Solid wall";
  if (tile.kind === 3) {
    const reason = tile.death_reason ? formatDeathReason(tile.death_reason) : null;
    return reason
      ? `Corpse · ${formatGlimLabel(tile.energy ?? 0)} · ${reason}`
      : `Corpse · ${formatGlimLabel(tile.energy ?? 0)}`;
  }
  if (tile.kind === 4) return `Energy node · ${formatGlimLabel(tile.energy ?? 0)}`;
  return "Unknown";
}
