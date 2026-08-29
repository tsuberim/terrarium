import { auth } from "./firebase";
import { apiRoot } from "./config";

export type Health = { status: string; tick_hz: number };
export type Me = { uid: string; credits: number };
export type Creature = {
  id: string;
  x: number;
  y: number;
  energy: number;
  health: number;
  max_health: number;
  owner_uid: string;
  /** WASM digest — used when sprite mode is hash. */
  program_hash?: string;
};
export type SimConfig = {
  r_vis: number;
  r_sig: number;
  corpse_energy: number;
  opcodes_per_tick: number;
  energy_per_opcode: number;
  move_extra: number;
  dig_extra: number;
  place_extra: number;
  hit_extra: number;
  signal_inbox_cap: number;
  max_health: number;
  hit_damage: number;
  health_regen: number;
  health_regen_cost: number;
};

export type WorldEvent =
  | {
      type: "signal";
      from_id: string;
      from_x: number;
      from_y: number;
      to_id?: string;
      byte: number;
      broadcast: boolean;
    }
  | {
      type: "death";
      creature_id: string;
      owner_uid: string;
      x: number;
      y: number;
      reason:
        | "energy_floor"
        | "out_of_energy"
        | "out_of_gas"
        | "empty_program"
        | "invalid_program"
        | "wasm_trap"
        | "out_of_vision"
        | "bad_direction"
        | "spawn_energy_too_low"
        | "signal_unknown_target"
        | "signal_out_of_range"
        | "suicide"
        | "spawn_failed"
        | "signal_failed"
        | "killed"
        | "eaten";
    }
  | { type: "spawn"; creature_id: string; parent_id: string; x: number; y: number }
  | {
      type: "hit";
      actor_id: string;
      victim_id: string;
      x: number;
      y: number;
      damage: number;
      victim_health: number;
    }
  | { type: "eat"; actor_id: string; x: number; y: number; energy: number };

export type DeathReason = Extract<WorldEvent, { type: "death" }>["reason"];

export type WorldTile = {
  x: number;
  y: number;
  kind: number;
  energy?: number;
  death_reason?: DeathReason;
};

export type World = {
  deploy_cost: number;
  corpse_energy: number;
  creatures: Creature[];
  tiles: WorldTile[];
};

async function api<T>(path: string, init: RequestInit = {}): Promise<T> {
  const headers = new Headers(init.headers);
  if (init.body && !headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json");
  }
  const user = auth.currentUser;
  if (user) {
    headers.set("Authorization", `Bearer ${await user.getIdToken()}`);
  }
  const res = await fetch(`${apiRoot()}${path}`, { ...init, headers });
  if (!res.ok) {
    const text = await res.text().catch(() => res.statusText);
    try {
      const body = JSON.parse(text) as { error?: string };
      throw new Error(body.error ?? text);
    } catch (e) {
      if (e instanceof Error && e.message !== text) throw e;
      throw new Error(text || res.statusText);
    }
  }
  return res.json() as Promise<T>;
}

export const getHealth = () => api<Health>("/health");
export const getMe = () => api<Me>("/v1/me");
export const getWorld = () => api<World>("/v1/world");
export const postFaucet = (amount: number) =>
  api<{ credits: number }>("/v1/faucet", {
    method: "POST",
    body: JSON.stringify({ amount }),
  });
export const postDeploy = (
  x: number,
  y: number,
  code: string,
  energy: number,
  wasmB64?: string,
) =>
  api<{ id: string; x: number; y: number; energy: number; credits: number }>("/v1/deploy", {
    method: "POST",
    body: JSON.stringify({ x, y, code, energy, wasm_b64: wasmB64 }),
  });

export const postClearWorld = () =>
  api<{ ok: boolean }>("/v1/dev/clear-world", { method: "POST" });

export const getSimConfig = () => api<SimConfig>("/v1/dev/sim-config");

export const patchSimConfig = (config: SimConfig) =>
  api<SimConfig>("/v1/dev/sim-config", {
    method: "PATCH",
    body: JSON.stringify(config),
  });
