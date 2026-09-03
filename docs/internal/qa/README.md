# Terrarium QA Framework

Local QA for Terrarium uses **one shared contract** (test hooks + state bridge + scenarios) and **three runners** that exercise it differently.

| Runner | Command | Use when |
|--------|---------|----------|
| **API smoke** | `npm run smoke` | Fast backend check — auth, compile, sandbox, deploy. No browser. |
| **Playwright** | `npm run e2e` | Repeatable UI flows, regression, CI. |
| **Cursor browser** | Agent skill `.cursor/skills/browser-qa` | Exploratory manual QA, layout, new flows. |

**Principle:** instrument the app once; runners differ only in how they drive the browser.

Dev workflow: [../workflow/README.md](../workflow/README.md).

---

## Quick start

```bash
./scripts/setup-dev.sh   # once
./scripts/dev.sh         # terminal 1 — auth emu, API, compile worker, Vite

npm run smoke               # terminal 2 — headless API smoke (works today)
```

Open **http://localhost:5173** — with auth emulator enabled, the app auto-signs in as `qa@terrarium.dev`.

For agent/browser QA, see [Browser QA (Cursor)](#browser-qa-cursor) below.

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Runners                                                │
│    npm run smoke          → scripts/api-smoke.sh (curl)     │
│    npm run e2e      → Playwright (apps/skin/e2e/)    │
│    Cursor browser MCP  → exploratory (agent skill)      │
├─────────────────────────────────────────────────────────┤
│  Shared contract                                        │
│    data-testid="e2e-*"   stable selectors                │
│    window.__TERRARIUM_E2E__   runtime state bridge      │
│    docs/internal/qa/scenarios/*.yaml  human + machine-readable flows  │
├─────────────────────────────────────────────────────────┤
│  Dev stack (./scripts/dev.sh)                           │
│    :9099  Firebase Auth Emulator                        │
│    :8080  terrarium-server (FIREBASE_AUTH_EMULATOR_HOST)│
│    :8081  compile-worker (body_wrap)                    │
│    :5173  Vite UI                                       │
└─────────────────────────────────────────────────────────┘
```

---

## Dev environment

### Services

| Port | Service | Health |
|------|---------|--------|
| 5173 | Vite UI | http://localhost:5173 |
| 8080 | API | http://127.0.0.1:8080/api/health |
| 8081 | Compile worker | http://127.0.0.1:8081/health — must return `"body_wrap": true` |
| 9099 | Auth emulator | REST sign-in (see below) |
| 4000 | Emulator UI | http://127.0.0.1:4000/auth |

### Environment variables

**Server** (set by `scripts/dev.sh`):

| Variable | Default | Purpose |
|----------|---------|---------|
| `FIREBASE_AUTH_EMULATOR_HOST` | `127.0.0.1:9099` | Server accepts emulator JWTs |
| `COMPILE_WORKER_URL` | `http://127.0.0.1:8081` | Rust compile proxy |
| `FAUCET_ENABLED` | `true` | Dev glims top-up |

**Frontend** (`apps/skin/.env.local`, set by `scripts/setup-dev.sh`):

| Variable | Default | Purpose |
|----------|---------|---------|
| `VITE_USE_AUTH_EMULATOR` | `true` | Connect Firebase client to `:9099` |
| `VITE_E2E_HOOKS` | `true` | Auto sign-in, auto-open Studio, expose QA bridge |

Disable QA conveniences: `VITE_E2E_HOOKS=false` or `VITE_USE_AUTH_EMULATOR=false`.

### QA user (auth emulator)

| Field | Value |
|-------|-------|
| Email | `qa@terrarium.dev` |
| Password | `qa-terrarium` |

Created on first sign-in. Same credentials used by `scripts/api-smoke.sh` and Playwright helpers.

---

## API smoke (`npm run smoke`)

Script: [`scripts/api-smoke.sh`](../scripts/api-smoke.sh)

Headless curl flow — no browser, no Vite required (but dev stack must be running):

1. Health checks (API + compile worker)
2. Auth emulator sign-in → Bearer token
3. `GET /v1/me`
4. `POST /v1/compile` (default Rust source)
5. `POST /v1/sandbox/run` (`open_field`, 20 ticks)
6. `POST /v1/deploy` (default on) — finds empty cell, faucets glims if needed

**Options:**

```bash
SMOKE_DEPLOY=0 npm run smoke          # skip deploy
SMOKE_DEPLOY_X=40 SMOKE_DEPLOY_Y=40 npm run smoke   # pin deploy cell (default 32,32)
SMOKE_EMAIL=... QA_PASSWORD=... npm run smoke
```

---

## App instrumentation

These hooks let Playwright and the Cursor browser agent interact reliably without brittle DOM guessing.

### QA mode

When `e2eHooksEnabled()` is true (local dev + auth emulator + `VITE_E2E_HOOKS !== false`):

- Auto sign-in as QA user
- Auto-open Creature Studio after auth
- Publish runtime state on `window.__TERRARIUM_E2E__`
- Set `document.body[data-e2e-ready="true"]` when idle and ready for interaction

### State bridge

```ts
window.__TERRARIUM_E2E__.getState()
// → { ready, signedIn, studioOpen, deployCell, deployDialogOpen,
//     credits, testing, wasmReady, playback, error }

window.__TERRARIUM_E2E__.waitFor(
  (s) => !s.testing && s.wasmReady,
  30_000
)
```

Agents read state via Cursor `browser_cdp` → `Runtime.evaluate`. Playwright uses a page helper.

### Test ID convention

Prefix: `data-testid="qa-{area}-{control}"`

| Area | IDs |
|------|-----|
| HUD | `e2e-hud-studio`, `e2e-hud-faucet`, `e2e-hud-sign-in`, `e2e-hud-sign-out`, `e2e-hud-jump` |
| Studio | `e2e-studio-test`, `e2e-studio-play`, `e2e-studio-stop`, `e2e-studio-deploy`, `e2e-studio-close` |
| Deploy dialog | `e2e-deploy-confirm`, `e2e-deploy-cancel`, `e2e-deploy-extra`, `e2e-deploy-location` |
| World | `e2e-world-map` |

Implementation targets: `HudOverlay.tsx`, `CreatureStudio.tsx`, `WorldCanvas.tsx`.

---

## Scenarios

YAML flows in [scenarios/](scenarios/) describe end-user journeys.

- **Playwright** — spec file per scenario (or shared runner parsing YAML)
- **Cursor agent** — follows steps manually using testids + QA bridge
- **API smoke** — covers backend subset without UI steps

See [scenarios/README.md](scenarios/README.md) for the schema.

| Scenario | File | Covers |
|----------|------|--------|
| Compile + sandbox | `studio-compile-test.yaml` | Studio, Test, Play |
| Deploy creature | `deploy-creature.yaml` | Map pick, faucet, deploy |
| Auth gate | `signed-out-gate.yaml` | Studio hidden when signed out |

---

## Browser QA (Cursor)

For agents doing exploratory manual QA in the Cursor IDE browser tab.

**Skill:** [`.cursor/skills/browser-qa/SKILL.md`](../.cursor/skills/browser-qa/SKILL.md)

**Typical flow:**

1. Confirm dev stack is up (`npm run preflight` when available, or curl health endpoints)
2. Navigate to http://localhost:5173
3. Read `window.__TERRARIUM_E2E__.getState()` — verify `signedIn`, `studioOpen`
4. Click via snapshot refs or `data-testid` elements
5. Map clicks: screenshot first, click `e2e-world-map` area (right of studio pane)
6. Deploy: pick cell on map → open deploy → confirm
7. Run `npm run smoke` to confirm API layer still healthy

**Known pitfalls:**

- Studio **closed** → shell has `pointer-events: none`; open Studio first (`e2e-hud-studio`)
- Deploy modal is centered — map clicks must land **outside** the panel bbox
- `browser_mouse_click_xy` requires a fresh screenshot first
- Prefer MCP `browser_click` over CDP synthetic events (may be blocked)

---

## Playwright

Location: `apps/skin/e2e/`

```bash
npm run e2e    # preflight + playwright test
```

- Base URL: `http://localhost:5173`
- Auth: QA mode auto sign-in (same as agent)
- Helpers: `e2e/helpers/e2e-bridge.ts`, `e2e/helpers/run-scenario.ts`
- Specs: `e2e/scenarios.spec.ts` — runs all `docs/internal/qa/scenarios/*.yaml`

**CI:** `test / smoke` runs `./scripts/ci-api-smoke.sh`; `test / e2e` runs `./scripts/ci-e2e.sh`. Both required on `main`.

---

## npm scripts

| Script | Status | Description |
|--------|--------|-------------|
| `npm run smoke` | **Live** | API smoke (`scripts/api-smoke.sh`) |
| `npm run preflight` | **Live** | Check all dev services before browser QA |
| `npm run e2e` | **Live** | Preflight + Playwright |
| `npm run test:integration` | **Live** | smoke then e2e |

---

## Implementation status

| Component | Status |
|-----------|--------|
| Auth emulator + auto sign-in | Done |
| `scripts/api-smoke.sh` | Done |
| `VITE_E2E_HOOKS` + e2e bridge | Done |
| `data-testid` hooks | Done |
| `scripts/stack-preflight.sh` | Done |
| Playwright e2e | Done |
| Scenario YAML files | Done |
| Agent skill | Done |

---

## Out of scope (v1)

- E2e hooks in production/staging
- `/api/v1/qa/state` HTTP endpoint (window bridge is enough)
- Visual regression / screenshot diffing
