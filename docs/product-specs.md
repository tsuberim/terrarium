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

**Status: in progress** (Firebase Auth + platform architecture folded in)

Players hold **credits** (account balance). Spawning a creature spends credits at a 1:1 rate with kernel mass — spawn is cash-in. The kernel's closed-box ledger still applies inside the world; credits are the rail that pays for `spawn_cell_at`.

### Surfaces

| Surface | What it is |
| --- | --- |
| **Skin** | Fullscreen retro camera. Destination: Firebase Hosting client, WebSocket viewer to always-on host. No billing chrome. |
| **Dashboard** | Firebase Hosting SPA. Firebase Auth for humans; client of `/dashboard/api/*` only. Balance, API tokens, billing stub. |
| **Public API** | HTTP JSON under `/v1/*`, authenticated with scoped API tokens minted after login. |
| **Host** | Always-on native `World` (PR #4). API spawn rail connects to it; kernel stays auth-free. |

### Identity

- **Humans:** Firebase Auth → ID token on dashboard API calls. Account id = Firebase uid.
- **Machines:** API tokens (`trm_…`), minted from dashboard, hashed at rest, **scoped** (`spawn`, `read`).
- **Local/CI:** dev session tokens when Firebase is not configured (`FIREBASE_PROJECT_ID` unset).

### Credits ledger

- Each account has a non-negative integer **credit balance**.
- Every change is recorded in a ledger (faucet, spawn spend, future Stripe top-up).
- **Spawn cost** = requested `mass` (minimum 1). Insufficient balance → `402`.
- Credits map to kernel mass at spawn time; conservation inside the box is unchanged.

### API tokens

- Mint and revoke from the dashboard (requires Firebase or dev auth).
- Token shown **once** at mint time; stored as SHA-256 hash server-side.
- Scopes: `spawn` (required for `POST /v1/spawn`), `read` (required for `GET /v1/world/snapshot`). Mint defaults to both.
- Revoked tokens fail auth immediately.

### Public API (v1)

| Method | Path | Auth | Scope |
| --- | --- | --- | --- |
| `POST` | `/v1/spawn` | API token | `spawn` |
| `GET` | `/v1/world/snapshot` | API token | `read` |
| `POST` | `/v1/accounts` | none | dev only (disabled when Firebase configured) |

`POST /v1/spawn` body: `{ "mass": 100, "x": 0, "y": 0, "program": "optional text" }`.

Responses: `401` bad/missing token or scope, `402` insufficient credits, `403` faucet disabled (prod), `400` invalid input.

### Dashboard (v1)

- Sign in with Firebase Auth (or dev sign-in locally).
- View credit balance and environment.
- Mint / list / revoke scoped API tokens.
- **Billing / top-up:** stub — Stripe coming soon.
- **Free credit faucet** (staging and local/dev only): hidden/disabled in production.

### Staging vs production — free mint

| Environment | `TERRARIUM_ENV` | Free mint |
| --- | --- | --- |
| Local / dev | `local` or `development` | **On** |
| Staging | `staging` | **On** |
| Production | `production` | **Off** — `403` |

### Out of scope (this milestone)

- Live Stripe checkout (seam only).
- Cash-out / attach / split.
- Full API→host WebSocket integration (architecture described; API still in-process world for v1).
- Firebase Hosting deploy workflows (config only; GCS workflows remain until switched).
