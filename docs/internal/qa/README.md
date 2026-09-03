# Terrarium QA Framework

Local QA for Terrarium uses **one shared contract** (test hooks + state bridge + scenarios) and **three runners** that exercise it differently.

| Runner | Command | Use when |
|--------|---------|----------|
| **API smoke** | `npm run qa` | Fast backend check — auth, compile, sandbox, deploy. No browser. |
| **Playwright** | `npm run qa:e2e` | Repeatable UI flows, regression, CI. |
| **Cursor browser** | Agent skill `.cursor/skills/browser-qa` | Exploratory manual QA, layout, new flows. |

**Principle:** instrument the app once; runners differ only in how they drive the browser.

Dev workflow: [../workflow/README.md](../workflow/README.md).

---

## Quick start

```bash
./scripts/setup-dev.sh   # once
./scripts/dev.sh         # terminal 1 — auth emu, API, compile worker, Vite

npm run qa               # terminal 2 — headless API smoke (works today)
```

Open **http://localhost:5173** — with auth emulator enabled, the app auto-signs in as `qa@terrarium.dev`.

For agent/browser QA, see [Browser QA (Cursor)](#browser-qa-cursor) below.

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Runners                                                │
│    npm run qa          → scripts/qa-smoke.sh (curl)     │
│    npm run qa:e2e      → Playwright (apps/skin/e2e/)    │
│    Cursor browser MCP  → exploratory (agent skill)      │
├─────────────────────────────────────────────────────────┤
│  Shared contract                                        │
│    data-testid="qa-*"   stable selectors                │
│    window.__TERRARIUM_QA__   runtime state bridge      │
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
| `VITE_QA_MODE` | `true` | Auto sign-in, auto-open Studio, expose QA bridge |

Disable QA conveniences: `VITE_QA_MODE=false` or `VITE_USE_AUTH_EMULATOR=false`.

### QA user (auth emulator)

| Field | Value |
|-------|-------|
| Email | `qa@terrarium.dev` |
| Password | `qa-terrarium` |

Created on first sign-in. Same credentials used by `scripts/qa-smoke.sh` and Playwright helpers.

---

## API smoke (`npm run qa`)

Script: [`scripts/qa-smoke.sh`](../scripts/qa-smoke.sh)

Headless curl flow — no browser, no Vite required (but dev stack must be running):

1. Health checks (API + compile worker)
2. Auth emulator sign-in → Bearer token
3. `GET /v1/me`
4. `POST /v1/compile` (default Rust source)
5. `POST /v1/sandbox/run` (`open_field`, 20 ticks)
6. `POST /v1/deploy` (default on) — finds empty cell, faucets glims if needed

**Options:**

```bash
QA_DEPLOY=0 npm run qa          # skip deploy
QA_DEPLOY_X=40 QA_DEPLOY_Y=40 npm run qa   # pin deploy cell (default 32,32)
QA_EMAIL=... QA_PASSWORD=... npm run qa
```

---

## App instrumentation

These hooks let Playwright and the Cursor browser agent interact reliably without brittle DOM guessing.

### QA mode

When `qaMode()` is true (local dev + auth emulator + `VITE_QA_MODE !== false`):

- Auto sign-in as QA user
- Auto-open Creature Studio after auth
- Publish runtime state on `window.__TERRARIUM_QA__`
- Set `document.body[data-qa-ready="true"]` when idle and ready for interaction

### State bridge

```ts
window.__TERRARIUM_QA__.getState()
// → { ready, signedIn, studioOpen, deployCell, deployDialogOpen,
//     credits, testing, wasmReady, playback, error }

window.__TERRARIUM_QA__.waitFor(
  (s) => !s.testing && s.wasmReady,
  30_000
)
```

Agents read state via Cursor `browser_cdp` → `Runtime.evaluate`. Playwright uses a page helper.

### Test ID convention

Prefix: `data-testid="qa-{area}-{control}"`

| Area | IDs |
|------|-----|
| HUD | `qa-hud-studio`, `qa-hud-faucet`, `qa-hud-sign-in`, `qa-hud-sign-out`, `qa-hud-jump` |
| Studio | `qa-studio-test`, `qa-studio-play`, `qa-studio-stop`, `qa-studio-deploy`, `qa-studio-close` |
| Deploy dialog | `qa-deploy-confirm`, `qa-deploy-cancel`, `qa-deploy-extra`, `qa-deploy-location` |
| World | `qa-world-map` |

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

1. Confirm dev stack is up (`npm run qa:preflight` when available, or curl health endpoints)
2. Navigate to http://localhost:5173
3. Read `window.__TERRARIUM_QA__.getState()` — verify `signedIn`, `studioOpen`
4. Click via snapshot refs or `data-testid` elements
5. Map clicks: screenshot first, click `qa-world-map` area (right of studio pane)
6. Deploy: pick cell on map → open deploy → confirm
7. Run `npm run qa` to confirm API layer still healthy

**Known pitfalls:**

- Studio **closed** → shell has `pointer-events: none`; open Studio first (`qa-hud-studio`)
- Deploy modal is centered — map clicks must land **outside** the panel bbox
- `browser_mouse_click_xy` requires a fresh screenshot first
- Prefer MCP `browser_click` over CDP synthetic events (may be blocked)

---

## Playwright

Location: `apps/skin/e2e/`

```bash
npm run qa:e2e    # preflight + playwright test
```

- Base URL: `http://localhost:5173`
- Auth: QA mode auto sign-in (same as agent)
- Helpers: `e2e/helpers/qa-bridge.ts`, `e2e/helpers/run-scenario.ts`
- Specs: `e2e/scenarios.spec.ts` — runs all `docs/internal/qa/scenarios/*.yaml`

**CI:** PRs run `./scripts/ci-e2e.sh` in the `e2e` job (`.github/workflows/reusable-test.yml`) — starts auth emulator, API, compile worker, Vite, then `npm run qa` + Playwright.

---

## npm scripts

| Script | Status | Description |
|--------|--------|-------------|
| `npm run qa` | **Live** | API smoke (`scripts/qa-smoke.sh`) |
| `npm run qa:preflight` | **Live** | Check all dev services before browser QA |
| `npm run qa:e2e` | **Live** | Preflight + Playwright |
| `npm run qa:all` | **Live** | `qa` then `qa:e2e` |

---

## Implementation status

| Component | Status |
|-----------|--------|
| Auth emulator + auto sign-in | Done |
| `scripts/qa-smoke.sh` | Done |
| `VITE_QA_MODE` + QA bridge | Done |
| `data-testid` hooks | Done |
| `scripts/qa-preflight.sh` | Done |
| Playwright e2e | Done |
| Scenario YAML files | Done |
| Agent skill | Done |

---

## Out of scope (v1)

- QA mode in production/staging
- `/api/v1/qa/state` HTTP endpoint (window bridge is enough)
- Visual regression / screenshot diffing
