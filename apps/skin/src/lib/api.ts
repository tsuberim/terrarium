import { auth } from "./firebase";
import { apiRoot } from "./config";

export type Health = { status: string; tick_hz: number };

export type ServerPowerStatus = {
  power_control_available: boolean;
  is_admin: boolean;
  min_instances: number | null;
  enabled: boolean | null;
};
export type Me = { uid: string; credits: number };

export type ApiKey = {
  id: string;
  name: string;
  prefix: string;
  created_at: string;
  last_used_at?: string;
};

export type MintApiKeyResponse = ApiKey & { secret: string };
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
  const text = await res.text().catch(() => "");
  if (!res.ok) {
    if (text) {
      try {
        const body = JSON.parse(text) as { error?: string };
        throw new Error(body.error ?? text);
      } catch (e) {
        if (e instanceof Error && e.message !== text) throw e;
      }
    }
    if (res.status === 404) {
      throw new Error("Endpoint not found — server may need an update");
    }
    throw new Error(text || res.statusText || `HTTP ${res.status}`);
  }
  if (res.status === 204 || !text) return undefined as T;
  try {
    return JSON.parse(text) as T;
  } catch {
    throw new Error("Invalid response from server");
  }
}

export const getHealth = () => api<Health>("/health");

/** Hit /health to cold-start Cloud Run (no auth; long timeout). */
export async function wakeServer(): Promise<Health> {
  const ctrl = new AbortController();
  const timer = window.setTimeout(() => ctrl.abort(), 120_000);
  try {
    const res = await fetch(`${apiRoot()}/health`, { signal: ctrl.signal });
    if (!res.ok) throw new Error("Server not responding");
    return (await res.json()) as Health;
  } finally {
    window.clearTimeout(timer);
  }
}

export const getServerPowerStatus = () => api<ServerPowerStatus>("/v1/admin/server-power");

export const postServerPower = (enabled: boolean) =>
  api<{ ok: boolean; enabled: boolean; min_instances: number }>("/v1/admin/server-power", {
    method: "POST",
    body: JSON.stringify({ enabled }),
  });
export const getMe = () => api<Me>("/v1/me");

export const getApiKeys = () => api<{ keys: ApiKey[] }>("/v1/api-keys");

export const mintApiKey = (name?: string) =>
  api<MintApiKeyResponse>("/v1/api-keys", {
    method: "POST",
    body: JSON.stringify({ name: name ?? "" }),
  });

export const deleteApiKey = (id: string) =>
  api<void>(`/v1/api-keys/${id}`, { method: "DELETE" });

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
