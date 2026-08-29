import { auth } from "./firebase";
import { apiRoot } from "./config";

export type Health = { status: string; tick_hz: number };
export type Me = { uid: string; credits: number };
export type Creature = { id: string; x: number; y: number; energy: number; owner_uid: string };
export type WorldTile = { x: number; y: number; kind: number; energy?: number };
export type World = { deploy_cost: number; creatures: Creature[]; tiles: WorldTile[] };

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
export const postDeploy = (x: number, y: number, code: string) =>
  api<{ id: string; x: number; y: number; energy: number; credits: number }>("/v1/deploy", {
    method: "POST",
    body: JSON.stringify({ x, y, code }),
  });
