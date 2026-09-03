# Product requirements (PRD)

Living product specification for Terrarium. Strategic vision: [vision.md](vision.md). System design: [../engineering/architecture.md](../engineering/architecture.md).

**Maintenance:** update this file **before** code changes; keep in sync through merge. Status labels: **shipped**, **dev-only**, **in progress**, **planned**. See [PRINCIPLES.md](../PRINCIPLES.md).

---

## 1. Product summary

Terrarium is a **persistent 2D hex-world simulation** where players deploy **programmable creatures** (WASM) that compete for energy. It is a simulation platform first — not a scored game with factions or win conditions.

| Pillar | Requirement |
|--------|-------------|
| Persistence | World runs continuously at 2 Hz; state survives restarts when DB is file-backed |
| Programmability | Creatures run user WASM each tick; code immutable after deploy |
| Economy | Energy = glims; paid deploy imports energy; world is deflationary |
| Spectating | Anyone can watch live; creature source is private |
| Emergence | Simple primitives; complex behavior from code + competition |

**Prod URL:** https://terrarium-506917.web.app  
**Public docs:** https://terrarium.mintlify.app (source: [`docs/public/`](../public/))

---

## 2. Principles (non-negotiable for v1)

From [vision.md](vision.md):

- **Bare bones** — no new primitives until multiple behaviors require them
- **Deflationary sim** — action costs + death destroy energy; free mint capped at 50% of lifetime destruction ([../engineering/sim/energy-budget.md](../engineering/sim/energy-budget.md))
- **Real stakes** — death matters; code cannot be patched in-place
- **Emergence over mechanics** — no territories, factions, leaderboards, combat stats
- **Open spectating, private code** — no public WASM/source on the wire

---

## 3. Users & roles

| Role | Can do | Cannot do |
|------|--------|-----------|
| **Visitor** | View map, follow creatures, read event feed, jump/search coords | Deploy, Studio, faucet, API keys |
| **Signed-in player** | All visitor actions + Studio, deploy, faucet (dev), API keys, manage glims | Read others' source code |
| **Creature owner** | Edit own Rust in Studio before deploy; see own diagnostics | Change code after deploy |

---

## 4. World & simulation (player-visible)

### 4.1 Topology

| ID | Requirement | Status |
|----|-------------|--------|
| WORLD-1 | Sparse hex grid (axial q/r); no wrap-around | **shipped** |
| WORLD-2 | Single global world (no shards) | **shipped** |
| WORLD-3 | World clock 2 Hz for all clients | **shipped** |
| WORLD-4 | Client interpolates motion between ticks from WS `actions` | **shipped** |
| WORLD-5 | Local dev: persistent SQLite `data/terrarium.db` | **shipped** |
| WORLD-6 | Prod Cloud Run: ephemeral SQLite (resets on redeploy) | **shipped** (known limitation) |

### 4.2 Cell types

| Kind | sense value | Player-visible behavior |
|------|-------------|-------------------------|
| Empty | 0 | Passable |
| Solid | 1 | Impassable; from dig/place |
| Creature | 2 | Via sense only |
| Corpse | 3 | Death tile; must eat for energy (~80% of creature energy) |
| Food | 4 | Budgeted free energy on map |

### 4.3 Seeded ecosystem

| ID | Requirement | Status |
|----|-------------|--------|
| ECO-1 | World seeds predators/prey/hawks when DB empty (`SEED_ECOSYSTEM`) | **shipped** |
| ECO-2 | Prey alarm signal `0x01` attracts hawks/scavengers | **shipped** |
| ECO-3 | Predator hunt ping `0x02` while chasing | **shipped** |

---

## 5. Creatures (sim rules → player expectations)

### 5.1 Programming model

| ID | Requirement | Status |
|----|-------------|--------|
| CRE-1 | Code immutable after deploy or spawn | **shipped** |
| CRE-2 | Code private on wire; only owner sees source in Studio | **shipped** |
| CRE-3 | At most **one action per tick** (move, rotate, eat, hit, dig, place, spawn, signal, suicide) | **shipped** |
| CRE-4 | `sleep` is free; creature chooses when to run WASM | **shipped** |
| CRE-5 | Facing 0–5 (E, NE, NW, W, SW, SE) | **shipped** |
| CRE-6 | Forward actions only on adjacent cell in facing direction | **shipped** |

### 5.2 Actions

| Action | Behavior |
|--------|----------|
| Move | Step forward if empty |
| Rotate | Turn in place (costs energy) |
| Eat | Forward corpse or food |
| Hit | Damage forward creature |
| Dig / Place | Modify forward cell |
| Spawn | Bud clone forward (parent pays energy) |
| Signal | Broadcast or directed byte within `r_sig` |
| Sleep / Suicide | Sleep free; suicide returns energy to owner |

### 5.3 Sensing

| ID | Requirement | Status |
|----|-------------|--------|
| SENSE-1 | `sense(dq,dr)` within `r_vis` (default 5) and frontal cone `vis_half_arc` (default ±60°) | **shipped** |
| SENSE-2 | Out of range/cone → empty read (no trap) | **shipped** |
| SENSE-3 | Sense returns kind, energy, health, facing for creatures | **shipped** |
| SENSE-4 | `recv` for incoming signals | **shipped** |

---

## 6. Economy (glims / credits / energy)

Display unit: **glims** (◆). Internal scale: `GLIM_SCALE = 100_000` (= sim `ENERGY_SCALE`).

| ID | Requirement | Status |
|----|-------------|--------|
| ECON-1 | HUD shows signed-in user's glim balance | **shipped** |
| ECON-2 | Deploy imports energy from credits 1:1 | **shipped** |
| ECON-3 | Deploy cost = `corpse_energy` (base) + `extra` (minimum extra = `deploy_cost`, typically 100 glims) | **shipped** |
| ECON-4 | Insufficient glims blocks deploy with clear error in modal | **shipped** |
| ECON-5 | Dev faucet: +100 glims per click when `FAUCET_ENABLED` | **shipped** dev-only |
| ECON-6 | Death: ~80% to corpse tile, ~20% destroyed | **shipped** |
| ECON-7 | Free food mint gated by energy ledger (2:1 destroy ratio) | **shipped** |
| ECON-8 | Paid real-money credits / cash-out | **planned** (open question) |

### Deploy vs spawn

| Path | Paid by | Code |
|------|---------|------|
| **Deploy** (human) | Account glims | User WASM from Studio |
| **Spawn** (creature) | Parent energy | Copy of parent code |

---

## 7. Authentication

| ID | Requirement | Status |
|----|-------------|--------|
| AUTH-1 | Studio, deploy, faucet, API keys require sign-in | **shipped** |
| AUTH-2 | Prod: Google sign-in via Firebase Auth popup | **shipped** |
| AUTH-3 | Local: Firebase Auth Emulator on `:9099` | **shipped** |
| AUTH-4 | Local QA: auto sign-in as `qa@terrarium.dev` / `qa-terrarium` when emulator + `VITE_QA_MODE` | **shipped** dev-only |
| AUTH-5 | Sign out clears studio shell, deploy cell, deploy dialog | **shipped** |
| AUTH-6 | API accepts Firebase JWT or API keys (Bearer) | **shipped** |
| AUTH-7 | Server validates emulator JWTs when `FIREBASE_AUTH_EMULATOR_HOST` set | **shipped** dev-only |

---

## 8. Spectator / world view (HUD)

| ID | Requirement | Status |
|----|-------------|--------|
| VIEW-1 | Real-time hex map with creatures and tiles via WebSocket | **shipped** |
| VIEW-2 | God view vs follow-creature camera (Map / Follow toggle) | **shipped** |
| VIEW-3 | Jump/search dialog (⌕): coords or creature id | **shipped** |
| VIEW-4 | Hover cell: coordinates + occupancy summary | **shipped** |
| VIEW-5 | Status bar: contextual hints (deploy, studio, jump, errors) | **shipped** |
| VIEW-6 | Death notice for own creature or followed creature (6s toast) | **shipped** |
| VIEW-7 | Event feed (signals, deaths, etc.) | **shipped** |
| VIEW-8 | Online indicator (WS connected) | **shipped** |
| VIEW-9 | My creatures list in HUD when signed in (click to follow) | **shipped** |
| VIEW-10 | Copy coordinates button when cell selected/hovered | **shipped** |
| VIEW-11 | Dev panel (sim config tuning) | **shipped** dev-only |

### Camera & URL state

| ID | Requirement | Status |
|----|-------------|--------|
| URL-1 | Persist view, zoom, follow, studio open, deploy cell in URL + localStorage | **shipped** |
| URL-2 | `?studio=1`, `?x=&y=` for studio + deploy focus | **shipped** |

---

## 9. Creature Studio

**Gate:** signed-in only. Studio hidden and non-interactive when signed out.

| ID | Requirement | Status |
|----|-------------|--------|
| STU-1 | Slide-in panel from left; resizable width (persisted `studioWidthPct`) | **shipped** |
| STU-2 | Resizable code/preview split (persisted `studioCodeHeightPct`) | **shipped** |
| STU-3 | Rust Monaco editor with default example source | **shipped** |
| STU-4 | Default source: `move_forward` body + `---` + `#[terrarium::scenario]` blocks | **shipped** |
| STU-5 | Live compile diagnostics (debounced) while editing | **shipped** |
| STU-6 | **Test** — compile all scenarios, sandbox replay loop | **shipped** |
| STU-7 | Sandbox preview: mini WorldCanvas, play/pause/stop, tick scrubber | **shipped** |
| STU-8 | Sandbox stats: tick, energy, spent, per-tick avg | **shipped** |
| STU-9 | Upload `.wasm` (skips compile; max size enforced) | **shipped** |
| STU-10 | **Deploy** button with rocket icon | **shipped** |
| STU-11 | Close studio (Escape when deploy dialog closed) | **shipped** |
| STU-12 | Closed studio shell: `pointer-events: none` (must open to interact) | **shipped** |
| STU-13 | QA mode: auto-open studio on load | **shipped** dev-only |
| STU-14 | Docs link to Mintlify from toolbar | **shipped** |

### Default Rust source contract

```rust
let _ = move_forward();
---
#[terrarium::scenario]
fn open_field() {}

#[terrarium::scenario(wall_ahead)]
fn wall_blocked() {}
```

Compile-worker wraps body in SDK template (`body_wrap: true` on worker health).

---

## 10. Deploy flow

| ID | Requirement | Status |
|----|-------------|--------|
| DEP-1 | Pick deploy cell by clicking map (crosshair when pick mode active) | **shipped** |
| DEP-2 | Pick works when studio visible OR deploy dialog open | **shipped** |
| DEP-3 | Deploy modal centered on screen (portal to `document.body`) | **shipped** |
| DEP-4 | Modal backdrop `pointer-events: none` — clicks pass through to map | **shipped** |
| DEP-5 | Modal panel shows location `(x,y)`, base corpse cost, extra glims, total | **shipped** |
| DEP-6 | Deploy confirm disabled until valid cell + enough glims + min extra | **shipped** |
| DEP-7 | Deploy sends WASM + code label to `POST /v1/deploy` | **shipped** |
| DEP-8 | Server rejects occupied/solid cells | **shipped** |
| DEP-9 | Successful deploy debits glims, closes dialog, clears deploy cell | **shipped** |
| DEP-10 | Recommended UX: pick map cell **before** opening deploy dialog | **shipped** (QA note) |

---

## 11. Compile & sandbox pipeline

| ID | Requirement | Status |
|----|-------------|--------|
| CMP-1 | Isolated compile-worker service (`:8081`) | **shipped** |
| CMP-2 | Server proxies `POST /v1/compile` to worker | **shipped** |
| CMP-3 | Worker health exposes `body_wrap: true` | **shipped** |
| CMP-4 | `POST /v1/sandbox/run` — WASM + scenario id + ticks → frames | **shipped** |
| CMP-5 | Scenarios parsed from `#[terrarium::scenario]` in source | **shipped** |
| CMP-6 | Compile-worker also deployable to Cloud Run (prod compile path) | **shipped** |

---

## 12. API keys

| ID | Requirement | Status |
|----|-------------|--------|
| KEY-1 | Signed-in users mint/list/revoke API keys (HUD → Keys dialog) | **shipped** |
| KEY-2 | Keys authenticate same REST API as Firebase JWT | **shipped** |
| KEY-3 | Secret shown once on mint | **shipped** |

---

## 13. REST & WebSocket API (product surface)

| Endpoint | Auth | Purpose |
|----------|------|---------|
| `GET /v1/world` | Optional | Snapshot: creatures, tiles, costs |
| `GET /v1/world/ws` | Optional | Live deltas + actions + events |
| `GET /v1/me` | Required | UID + credits |
| `POST /v1/deploy` | Required | Place creature |
| `POST /v1/compile` | Required | Rust → WASM |
| `POST /v1/sandbox/run` | Required | Scenario replay |
| `POST /v1/faucet` | Required | Dev credits top-up |
| `GET/POST/DELETE /v1/api-keys` | Required | Key management |
| `GET /health` | Public | Liveness |
| `GET /docs` | Public | Scalar API docs |

OpenAPI: `/api/openapi.json`

---

## 14. Client rendering (UX contract)

| ID | Requirement | Status |
|----|-------------|--------|
| REN-1 | Sim state from WS merged immediately (HUD, deploy checks) | **shipped** |
| REN-2 | Display layer interpolates from `actions` + `events` | **shipped** |
| REN-3 | Eat FX holds removed tile until animation completes | **shipped** |
| REN-4 | Death ghosts from rich death events | **shipped** |
| REN-5 | Creature sprites vary by health; sense/vision overlays in follow/god | **shipped** |

---

## 15. Explicit non-goals (v1)

Do not implement without explicit product decision:

- Multi-region / sharded worlds
- Factions, territories, opt-in PvP rules
- Scoreboards / leaderboards
- Torus / wrap-around map edges
- Public inspection of creature source or WASM hashes
- In-place code updates after deploy
- Staging environment (prod only today)

---

## 16. Open product questions

| Topic | Notes |
|-------|-------|
| Sleep interrupt conditions | Undecided |
| Payment provider / cash-out | Undecided |
| Corpse decay over time | Undecided |
| Energy cost tuning at scale | Undecided |
| Persistent prod database | Cloud Run ephemeral today — see [../engineering/tech-debt.md](../engineering/tech-debt.md) TD-INF-1 |
| Prod compile-worker always-on | Optional separate service |

---

## 17. Local QA & testing

Dev-only automation; not exposed in prod. Detail: [../qa/README.md](../qa/README.md).

| ID | Requirement | Status |
|----|-------------|--------|
| QA-1 | `npm run qa` — headless API smoke (auth, compile, sandbox, deploy) | **shipped** dev-only |
| QA-2 | `npm run qa:e2e` — Playwright runs every `docs/internal/qa/scenarios/*.yaml` | **shipped** |
| QA-3 | `VITE_QA_MODE` exposes `window.__TERRARIUM_QA__` + `data-testid="qa-*"` | **shipped** dev-only |
| QA-4 | `npm run qa:preflight` verifies dev stack before browser/e2e | **shipped** |
| QA-5 | CI `e2e` job runs `./scripts/ci-e2e.sh` (same smoke + Playwright) | **shipped** |
| QA-6 | API smoke deploy uses pinned cell (`QA_DEPLOY_X` / `QA_DEPLOY_Y`, default 32,32) with scan fallback | **shipped** |
| QA-7 | API smoke tops up credits in faucet-sized chunks when deploy cost exceeds one request | **shipped** |

Scenarios: studio compile+playback, deploy creature, signed-out auth gate.

---

## 18. Related docs

| Doc | Contents |
|-----|----------|
| [vision.md](vision.md) | Principles |
| [../engineering/architecture.md](../engineering/architecture.md) | Server, WS protocol |
| [../engineering/sim/energy-budget.md](../engineering/sim/energy-budget.md) | Economy math |
| [../engineering/sim/host-abi.md](../engineering/sim/host-abi.md) | Host ABI |
| [../ops/deploy.md](../ops/deploy.md) | Prod deploy |
| [../engineering/tech-debt.md](../engineering/tech-debt.md) | Eng debt |
| [../qa/README.md](../qa/README.md) | QA framework |
| [../../public/](../../public/) | Player Mintlify source |
