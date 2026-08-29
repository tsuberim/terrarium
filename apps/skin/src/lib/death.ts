import type { DeathReason, WorldEvent } from "./api";

export type DeathEvent = Extract<WorldEvent, { type: "death" }>;
export type { DeathReason };

const REASON_LABELS: Record<DeathReason, string> = {
  energy_floor: "hit energy floor",
  out_of_energy: "ran out of energy",
  out_of_gas: "ran out of gas",
  empty_program: "empty program",
  invalid_program: "invalid program",
  wasm_trap: "program trap",
  out_of_vision: "looked out of vision",
  bad_direction: "bad direction",
  spawn_energy_too_low: "spawn energy too low",
  signal_unknown_target: "signal to unknown target",
  signal_out_of_range: "signal out of range",
  suicide: "self-destructed",
  spawn_failed: "spawn failed",
  signal_failed: "signal failed",
  killed: "killed in combat",
  eaten: "eaten",
};

export function formatDeathReason(reason: DeathReason): string {
  return REASON_LABELS[reason] ?? reason.replaceAll("_", " ");
}

export function formatDeathNotice(event: DeathEvent): string {
  const id = event.creature_id.slice(0, 8);
  return `${id} died — ${formatDeathReason(event.reason)}`;
}
