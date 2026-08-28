# Product specs

Concrete product behaviour — what the game should look and feel like. Vision and architecture stay in their own docs; this file is the checklist for skin and UX work.

## How specs work

Slack `#product-requests` is intake. Drop a bullet there when you want something changed. Once we agree it is real work, the spec lands here — not in chat threads.

Read this file before coding a product change. When something ships or starts, update its status so the doc stays honest.

Status tags:

- **shipped** — in main, matches the spec below
- **in progress** — someone is building it
- **requested** — agreed direction, not done yet

## Fullscreen retro camera

**Status: shipped**

The skin is a fullscreen retro camera on the sim. No chrome, no dashboard.

- Pixelated look — chunky framebuffer, nearest-neighbor upscale (`image-rendering: pixelated`), CRT scanlines
- Minimalist — world fills the viewport; almost nothing else on screen
- No stats / HUD — no tick counter, mass totals, house burned, FPS, or other overlays on the raw sim
- Hideable program overlay — wander / chase / sit demos and the paste-a-program editor stay available, but tuck away so writing a creature program does not break the fullscreen feel

Kernel rules unchanged. The camera gets prettier; the box does not.

## Wrapping / toroidal open world

**Status: requested** (implementation on a separate branch)

Not a petri dish. The world wraps like a torus:

- Move off the right edge → pop in on the left (same y)
- Move off the left edge → pop in on the right
- Same for top and bottom

No hard walls that stop you at the rim. No circular "dish" boundary. Open world feel with finite wrap-around space.

Mass conservation and kernel verbs stay as in vision — this is geometry and camera framing, not new economy rules.

## Billing, API tokens, and public spawn

**Status: shipped**

Players hold **credits** (account balance). Spawning a creature spends credits at a 1:1 rate with kernel mass — spawn is cash-in. The kernel's closed-box ledger still applies inside the world; credits are the rail that pays for `spawn_cell_at`.

### Surfaces

| Surface | What it is |
| --- | --- |
| **Skin** | Fullscreen retro camera on the sim. Unchanged. No billing chrome. |
| **Dashboard** | Separate web UI at `/dashboard/` (served by the API). Balance, API tokens, billing/top-up stub. |
| **Public API** | HTTP JSON under `/v1/*`, authenticated with minted API tokens. |

### Credits ledger

- Each account has a non-negative integer **credit balance**.
- Every change is recorded in a ledger (faucet, spawn spend, future Stripe top-up).
- **Spawn cost** = requested `mass` (minimum 1). Insufficient balance → `402` with a clear error.
- Credits map to kernel mass at spawn time; conservation inside the box is unchanged.

### API tokens

- Mint and revoke from the dashboard.
- Token shown **once** at mint time (`trm_…` prefix); stored as SHA-256 hash server-side.
- Revoked tokens fail auth immediately.
- Public API uses `Authorization: Bearer <token>`.

### Public API (v1)

| Method | Path | Auth | Behaviour |
| --- | --- | --- | --- |
| `POST` | `/v1/accounts` | none | Create account; returns `account_id` + dashboard `session_token`. |
| `POST` | `/v1/spawn` | API token | Spend credits; spawn cell in the server world via kernel `spawn_cell_at`. |
| `GET` | `/v1/world/snapshot` | API token | Read-only world snapshot (tick, cells, mass totals). |

`POST /v1/spawn` body: `{ "mass": 100, "x": 0, "y": 0, "program": "optional text" }`.

Responses use clear JSON errors: `401` bad/missing token, `402` insufficient credits, `400` invalid input.

### Dashboard (v1)

- Create or resume account (session token in `localStorage`).
- View credit balance and environment name.
- Mint / list / revoke API tokens.
- **Billing / top-up**: honest stub — "Stripe checkout coming soon"; no live payment keys required.
- **Free credit faucet** (staging and local/dev only): button to mint test credits. Hidden/disabled in production.

### Staging vs production — free mint

| Environment | `TERRARIUM_ENV` | Free mint |
| --- | --- | --- |
| Local / dev | `local` or `development` | **On** — open faucet |
| Staging | `staging` | **On** — open faucet |
| Production | `production` | **Off** — faucet returns `403`; no free credits |

Production must never allow the faucet. Staging and local must allow it so API spawn can be QA'd without Stripe.

### Out of scope (v1)

- Live Stripe checkout (seam only).
- Cash-out verb / real-money withdrawal.
- Multiplayer / browser WASM talking to the server world (server runs kernel natively; skin stays local WASM for now).
- Attach/split verbs.
