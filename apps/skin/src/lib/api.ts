import { auth } from "./firebase";
import { apiRoot } from "./config";

export type Health = { status: string; tick_hz: number };

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
  /** Body facing 0–5 (E, NE, NW, W, SW, SE). */
  facing: number;
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
  rotate_extra: number;
  /** Frontal vision half-width in hex direction steps (1 = ±60°). */
  vis_half_arc: number;
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
      facing: number;
      energy: number;
      health: number;
      max_health: number;
    }
  | { type: "spawn"; creature_id: string; parent_id: string; parent_x: number; parent_y: number; x: number; y: number }
  | {
      type: "hit";
      actor_id: string;
      victim_id: string;
      x: number;
      y: number;
      damage: number;
      victim_health: number;
    }
  | { type: "eat"; actor_id: string; x: number; y: number; energy: number; tile_kind: number };

/** Explicit per-tick creature action from the sim (matches sim wire format). */
export type CreatureAction =
  | { kind: "move"; creature_id: string; from_x: number; from_y: number; to_x: number; to_y: number }
  | { kind: "rotate"; creature_id: string; from_facing: number; to_facing: number }
  | { kind: "eat"; creature_id: string; x: number; y: number }
  | { kind: "hit"; creature_id: string; x: number; y: number };

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

export type CompileResult = {
  ok: boolean;
  wasm_b64?: string;
  diagnostics: { level: string; message: string; line?: number; column?: number; area?: string }[];
};

export async function postCompile(language: string, source: string, tests: string): Promise<CompileResult> {
  const headers = new Headers({ "Content-Type": "application/json" });
  const user = auth.currentUser;
  if (user) {
    headers.set("Authorization", `Bearer ${await user.getIdToken()}`);
  }
  const res = await fetch(`${apiRoot()}/v1/compile`, {
    method: "POST",
    headers,
    body: JSON.stringify({ language, source, tests }),
  });
  const text = await res.text().catch(() => "");
  let body: CompileResult & { error?: string };
  try {
    body = text ? (JSON.parse(text) as CompileResult & { error?: string }) : { ok: false, diagnostics: [] };
  } catch {
    throw new Error(text || res.statusText || `HTTP ${res.status}`);
  }
  if (res.ok || (res.status === 400 && Array.isArray(body.diagnostics))) {
    return body;
  }
  const msg = body.diagnostics?.[0]?.message ?? body.error ?? "Compile failed";
  throw new Error(msg);
}

export const postSandboxRun = (wasmB64: string, test: import("./creatureEditor").TestSpec) =>
  api<import("./creatureEditor").SandboxResult>("/v1/sandbox/run", {
    method: "POST",
    body: JSON.stringify({ wasm_b64: wasmB64, test }),
  });

export const postClearWorld = () =>
  api<{ ok: boolean }>("/v1/dev/clear-world", { method: "POST" });

export const getSimConfig = () => api<SimConfig>("/v1/dev/sim-config");

export const patchSimConfig = (config: SimConfig) =>
  api<SimConfig>("/v1/dev/sim-config", {
    method: "PATCH",
    body: JSON.stringify(config),
  });
