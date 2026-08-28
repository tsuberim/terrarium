# Product specs

Concrete product behaviour — what the game should look and feel like. Vision and architecture stay in their own docs; this file is the checklist for skin and UX work.

## How specs work

Slack `#product-requests` is intake. Drop a bullet there when you want something changed. Once we agree it is real work, the spec lands here — not in chat threads.

Read this file before coding a product change. When something ships or starts, update its status so the doc stays honest.

Status tags:

- **shipped** — in main, matches the spec below
- **in progress** — someone is building it
- **requested** — agreed direction, not done yet

## Platform framing

**Status: shipped** (docs + initial surfaces)

Terrarium is a **platform**, not only a browser game:

| Layer | What it is |
| --- | --- |
| **Game world** | Auth-free kernel + always-on host. Mass conservation, verbs, toroidal geometry. |
| **Account / API** | Credits ledger, scoped API tokens, `POST /v1/spawn`, Firebase Auth for humans. |
| **Hosting** | Static browser shell (landing, play camera, console) on Firebase Hosting; Cloud Run host + API. |

Vision stays a programming game. The **product** is the platform around it — spawn rail, credits, dashboard, persistent host.

## Browser app shell

**Status: shipped** (v1 — landing, about, play, console; polish ongoing)

One product, sparse chrome, short copy. System fonts. No clutter.

| Surface | App | Role |
| --- | --- | --- |
| **Landing** | `apps/site` | Brief intro — what Terrarium is, why mass = money, link to play and console. |
| **About** | `apps/site/about.html` | Deeper read — kernel vs skin, verbs, torus, platform API. |
| **Play** | `apps/skin` | Fullscreen retro camera on the sim. Primary surface. No billing chrome. |
| **Console** | `apps/dashboard` | Firebase Auth, credits, API tokens, billing stub. |

Shared nav (`apps/shared/shell.css`, `links.js`) ties the surfaces together. Cross-app URLs are set via `<meta name="terrarium-*">` tags per deploy target (home, about, play, console).

Design rules:

- Elegant, minimalist — clean typography, concise sentences
- Play stays edge-to-edge; chrome floats, does not shrink the sim
- Console matches landing palette; no API contract changes for polish

## Fullscreen retro camera

**Status: shipped**

The skin is a fullscreen retro camera on the sim. No stats HUD.

- **Edge-to-edge viewport** — the torus fills the browser window (cover scale). No letterboxed frame, no inset margins shrinking the sim. Chrome (wordmark / program overlay) floats on top and must not reduce the drawable area.
- **Bigger world** — `WORLD_WIDTH` / `WORLD_HEIGHT` are 800_000 fixed-point units each. Wrapping stays; no circular boundary.
- **Higher-res pixel framebuffer** — 480px on the short axis, aspect-following long axis (PR #14). Nearest-neighbor upscale (`image-rendering: pixelated`); CRT scanlines optional overlay.
- Minimalist — almost nothing else on screen besides the sim
- No stats / HUD — no tick counter, mass totals, house burned, FPS, or other overlays on the raw sim
- Hideable program overlay — wander / chase / sit demos and the paste-a-program editor stay available, but tuck away so writing a creature program does not break the fullscreen feel
- Wordmark + console link in floating chrome; home/about via meta URLs

Kernel rules unchanged. The camera gets prettier; the box does not.

## Wrapping / toroidal open world

**Status: shipped**

Not a petri dish. The world wraps like a torus:

- Move off the right edge → pop in on the left (same y)
- Move off the left edge → pop in on the right
- Same for top and bottom

No hard walls that stop you at the rim. No circular "dish" boundary. Open world feel with finite wrap-around space.

Mass conservation and kernel verbs stay as in vision — this is geometry and camera framing, not new economy rules.

## Billing, API tokens, and public spawn

**Status: shipped** (API + dashboard + host + Firebase Hosting deploy on main)

Players hold **credits** (account balance). Spawning a creature spends credits at a 1:1 rate with kernel mass — spawn is cash-in. The kernel's closed-box ledger still applies inside the world; credits are the rail that pays for `spawn_cell_at`.

### Surfaces

| Surface | What it is |
| --- | --- |
| **Skin** | Fullscreen retro camera. Destination: Firebase Hosting client, WebSocket viewer to always-on host. No billing chrome. |
| **Dashboard** | Firebase Hosting SPA. Firebase Auth for humans; client of `/dashboard/api/*` only. Balance, API tokens, billing stub. |
| **Public API** | HTTP JSON under `/v1/*`, authenticated with scoped API tokens minted after login. |
| **Host** | Always-on native `World`. API spawn rail delegates via `/internal/spawn`. |

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

### Dashboard / console (v1)

- Sign in with Firebase Auth (or dev sign-in locally).
- View credit balance and environment.
- Mint / list / revoke scoped API tokens.
- **Billing / top-up:** stub — Stripe coming soon.
- **Free credit faucet** (staging and local/dev only): hidden/disabled in production.
- Shared product nav with landing and play.

### Staging vs production — free mint

| Environment | `TERRARIUM_ENV` | Free mint |
| --- | --- | --- |
| Local / dev | `local` or `development` | **On** |
| Staging | `staging` | **On** |
| Production | `production` | **Off** — `403` |

### Out of scope (this milestone)

- Live Stripe checkout (seam only).
- Cash-out / attach / split.
