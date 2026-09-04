import type { WorldEvent } from "./api";
import { formatDeathReason } from "./death";
import { formatGlimString } from "./glim";
import { shortId } from "./wireIds";

export type TimestampedEvent = WorldEvent & { at: number };

export function formatWorldEvent(event: WorldEvent): string {
  switch (event.type) {
    case "death":
      return `${shortId(event.creature_id)} died — ${formatDeathReason(event.reason)}`;
    case "hit":
      return `${shortId(event.actor_id)} hit ${shortId(event.victim_id)} (−${event.damage} hp)`;
    case "eat":
      return `${shortId(event.actor_id)} ate ${formatGlimString(event.energy)}`;
    case "spawn":
      return `${shortId(event.parent_id)} budded ${shortId(event.creature_id)}`;
    case "signal":
      if (event.broadcast) {
        return `${shortId(event.from_id)} broadcast`;
      }
      return `${shortId(event.from_id)} → ${event.to_id != null ? shortId(event.to_id) : "?"} sig`;
    default:
      return "world event";
  }
}

export function eventTone(event: WorldEvent): "combat" | "food" | "life" | "signal" | "neutral" {
  switch (event.type) {
    case "hit":
    case "death":
      return event.type === "death" && event.reason === "killed" ? "combat" : "life";
    case "eat":
      return "food";
    case "spawn":
      return "life";
    case "signal":
      return "signal";
    default:
      return "neutral";
  }
}
