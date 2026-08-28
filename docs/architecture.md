# Architecture

Terrarium is four layers plus identity. The **kernel** is physics and mass accounting only — no auth, no Stripe, no tokens. Everything human-facing sits outside it.

```
┌─────────────────────────────────────────────────────────────────┐
│  Firebase Hosting (destination)                                 │
│    apps/skin      — camera + program editor (static SPA)        │
│    apps/dashboard — billing, tokens, balance (static SPA)       │
└────────────┬───────────────────────────────┬────────────────────┘
             │ WebSocket / SSE               │ HTTPS JSON
             ▼                               ▼
┌────────────────────────┐      ┌─────────────────────────────────┐
│  Host (crates/host)    │◄────►│  API (crates/api)               │
│  always-on native World│      │  credits, tokens, spawn rail    │
│  ticks ~20 Hz          │      │  Firebase JWT + API token auth  │
└───────────┬────────────┘      └─────────────────────────────────┘
            │ links
            ▼
┌────────────────────────┐
│  Kernel (crates/kernel)│  deterministic physics, mass ledger
│  auth-free             │  spawn = cash-in; spend = house burn
└────────────────────────┘
```

**Destination:** one always-on native process owns the authoritative `World`. Browsers are viewers and control surfaces only. **Legacy (still on main today):** the skin can tick kernel WASM in-tab and static files deploy to GCS — both are transitional until host + Firebase Hosting land.

## Kernel

Rust crate: `crates/kernel`. Deterministic fixed-point 2D physics on a rectangular torus. Mass accounting is load-bearing.

Creatures run a tiny bytecode program inside the kernel (WASM guest modules can come later). Mass is their fuel. Guest code does not get free compute: cycles spent are mass spent, burned to the house.

The crate compiles natively (what the host and API link) and optionally to WASM (`scripts/build-wasm.sh`) for the legacy in-tab skin and tests. **The kernel knows nothing about Firebase, Stripe, API tokens, or credits.**

### Seven verbs

| Verb | Meaning |
| --- | --- |
| **thrust** | Move. Costs mass (burned to the house). |
| **sense** | Look. Costs mass (burned to the house). |
| **absorb** | Take inert mass into yourself. Explicit. Touching does not eat. |
| **dump** | Leave inert mass in the world (walls, shots, debris). Still in the box. |
| **attach** | Join cells into a body. |
| **split** | Divide a cell. Mass splits with it. |
| **cash out** | Mass leaves the box as money. |

Sleep is not a verb. Sleep is free.

`spend` bills thrust, sense, and other action: mass moves from the cell to `house_burned`. `dump_matter` / `absorb_matter` are internal transfers and must conserve `total_mass()`.

## Host (always-on game process)

Crate: `crates/host` (see open PR #4 — native host + WebSocket skin). **Not merged to main yet**; architecture assumes it is the sim process of record.

- Owns one `World`, seeds it, calls `tick` on a fixed interval whether or not a browser is open.
- Serves the skin static files (today from the same process; destination: Firebase Hosting with WS to host).
- Pushes world snapshots to clients over **WebSocket** (`/ws`); accepts `set_program`, `reset`.
- Scale-to-zero would wipe in-memory state, so production uses **min instances 1, max 1** on a cheap container (Cloud Run or equivalent — not a farm).

The host does **not** own billing. Spawn paid for with credits goes through the API, which calls into the shared world (today in-process on `crates/api`; later a call from API → host).

## Skin

Fullscreen retro **camera** — not the game. Destination: static app on **Firebase Hosting**, connecting to the host via WebSocket/SSE for snapshots. No sim ticking in the tab.

**Legacy today:** `apps/skin` loads kernel WASM and ticks locally; deployed to GCS buckets. That path is deprecated once host + Firebase Hosting are wired.

## Dashboard

Separate static SPA on **Firebase Hosting** (`apps/dashboard`). Client of the API only — not a second backend.

- **Firebase Auth** for humans (Google / email). Browser sends Firebase ID token as `Authorization: Bearer <jwt>` to `/dashboard/api/*`.
- Credit balance, mint/revoke **API tokens**, billing stub (Stripe seam).
- Staging/local **free-credit faucet** (disabled in production).

## API (credits, tokens, public spawn)

Crate: `crates/api`. Thin Axum + SQLite layer. Owns everything the kernel must not know about.

| Concern | Owner |
| --- | --- |
| Firebase ID token verify | API |
| Credit ledger (off-world money) | API |
| API token mint/revoke (hashed at rest) | API |
| `POST /v1/spawn` — credits → `spawn_cell_at` | API |
| Staging free-credit faucet | API |
| JWT / Stripe / Firebase inside kernel | **never** |

### Auth paths

**Humans (dashboard):** Firebase Auth → ID token → API verifies JWT (`FIREBASE_PROJECT_ID`, Google's x509 keys). Account id = Firebase `sub`. First login creates the credits row.

**Machines (public API):** API tokens minted from the dashboard after human login. Prefix `trm_…`, stored as SHA-256. **Scoped:** `spawn`, `read` (comma-separated in DB). Revoked tokens fail immediately.

**Local / CI without Firebase:** when `FIREBASE_PROJECT_ID` is unset, `TERRARIUM_DEV_AUTH=1` (default) allows legacy dev session tokens (`trm_sess_…`) from `POST /v1/accounts` so tests and local QA work without Firebase keys.

### Public API (v1)

| Method | Path | Auth | Scope |
| --- | --- | --- | --- |
| `POST` | `/v1/spawn` | API token | `spawn` |
| `GET` | `/v1/world/snapshot` | API token | `read` |
| `POST` | `/v1/accounts` | none | dev only — disabled when Firebase is configured |

Credits are money **outside** the box; mass is money **inside**. Spawn spends credits 1:1 with requested `mass`, then the API calls kernel `spawn_cell_at`.

### Economy seam

```
credits (SQLite)  ──spawn──►  World::spawn_cell_at  ──►  mass in the box
     ▲                              │
     │                              └── spawned_mass, total_mass, house_burned
  faucet / Stripe (later)
```

## Identity & hosting (destination)

| Piece | Technology | Notes |
| --- | --- | --- |
| Human identity | **Firebase Auth** | ID tokens to API; no passwords in our DB |
| Static clients | **Firebase Hosting** | Skin + dashboard SPAs |
| Always-on sim | **Host** (`crates/host`) | Native kernel, WebSocket |
| Credits / tokens / spawn rail | **API** (`crates/api`) | SQLite; merges into host process later |
| Legacy static deploy | GCS buckets | **TODO:** migrate workflows to Firebase Hosting (see `firebase.json`, `docs/environments.md`) |

Public Firebase web config (`apiKey`, `authDomain`, `projectId`) is not secret — served from `/dashboard/api/config` or checked-in example. Service account keys stay in `~/keys/` only.

## Mass accounting

The closed box is denominated in `Mass` (`u64`).

- `World::spawned_mass()` is total cash-in. Until cash-out exists: `spawned_mass == total_mass + house_burned`.
- `World::total_mass()` is living cells plus inert dumps.
- `World::house_burned()` is mass destroyed by acting and computing.
- `spawn_cell` / `spawn_cell_at` are cash-in.
- Cash-out (later) is the inverse of spawn.

`WORLD_WIDTH` and `WORLD_HEIGHT` define a rectangular torus. Tests in `crates/kernel` pin ledger identity, conservation, toroidal wrap, and sense costing.

## Why conservation matters

Mass is money. If the kernel mints or loses grams, the economy is a bug, not a game. The house burn is the only designed destruction. That is why the first code in the repo is a mass ledger, not a renderer — and why auth and billing stay out of it.
