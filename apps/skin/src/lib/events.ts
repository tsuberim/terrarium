import type { WorldEvent } from "./api";
import { formatDeathReason } from "./death";
import { formatGlimString } from "./glim";

export type TimestampedEvent = WorldEvent & { at: number };

export function formatWorldEvent(event: WorldEvent): string {
  switch (event.type) {
    case "death":
      return `${event.creature_id.slice(0, 8)} died — ${formatDeathReason(event.reason)}`;
    case "hit":
      return `${event.actor_id.slice(0, 8)} hit ${event.victim_id.slice(0, 8)} (−${event.damage} hp)`;
    case "eat":
      return `${event.actor_id.slice(0, 8)} ate ${formatGlimString(event.energy)}`;
    case "spawn":
      return `${event.parent_id.slice(0, 8)} budded ${event.creature_id.slice(0, 8)}`;
    case "signal":
      if (event.broadcast) {
        return `${event.from_id.slice(0, 8)} ping 0x${event.byte.toString(16).padStart(2, "0")}`;
      }
      return `${event.from_id.slice(0, 8)} → ${event.to_id?.slice(0, 8) ?? "?"} sig`;
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
