import type { CreatureAction, WorldEvent } from "./api";

/** Normalize u64 ids from sim wire (JSON number) to string keys. */
export function wireId(id: string | number | undefined | null): string {
  if (id == null) return "";
  return typeof id === "number" ? String(id) : id;
}

export function shortId(id: string | number): string {
  const s = wireId(id);
  return s.length <= 8 ? s : `${s.slice(0, 8)}…`;
}

export function normalizeWorldEvent(event: WorldEvent): WorldEvent {
  switch (event.type) {
    case "signal":
      return {
        ...event,
        from_id: wireId(event.from_id),
        to_id: event.to_id != null ? wireId(event.to_id) : undefined,
      };
    case "death":
      return { ...event, creature_id: wireId(event.creature_id) };
    case "spawn":
      return {
        ...event,
        creature_id: wireId(event.creature_id),
        parent_id: wireId(event.parent_id),
      };
    case "hit":
      return {
        ...event,
        actor_id: wireId(event.actor_id),
        victim_id: wireId(event.victim_id),
      };
    case "eat":
      return { ...event, actor_id: wireId(event.actor_id) };
    default:
      return event;
  }
}

export function normalizeCreatureAction(action: CreatureAction): CreatureAction {
  return { ...action, creature_id: wireId(action.creature_id) };
}
