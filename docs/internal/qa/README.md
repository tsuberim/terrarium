# Terrarium QA

One contract (e2e hooks + scenarios), three runners.

| Runner | Command | Use when |
|--------|---------|----------|
| **API smoke** | `npm run smoke` | Backend — auth, compile, sandbox, deploy. No browser. |
| **Playwright** | `npm run e2e` | UI regression, CI. |
| **Cursor browser** | `.cursor/skills/browser-qa` | Exploratory manual QA. |

Dev stack and env vars: [../workflow/setup.md](../workflow/setup.md). Dev loop: [../workflow/README.md](../workflow/README.md).

---

## Quick start

```bash
./scripts/setup-dev.sh && ./scripts/dev.sh   # terminal 1
npm run smoke                                 # terminal 2
```

Auto sign-in as `qa@terrarium.dev` when auth emulator + `VITE_E2E_HOOKS` are on.

---

## Architecture

```
Runners:  smoke (curl) · e2e (Playwright) · browser MCP (agent skill)
Contract: data-testid="e2e-*" · window.__TERRARIUM_E2E__ · scenarios/*.yaml
Stack:    :9099 auth · :8080 API · :8081 compile · :5173 Vite

With `VITE_E2E_HOOKS=true`, Creature Studio skips the debounced compile-on-open check so scenario steps (Run test / Run all) own compile without racing the worker.
```

---

## E2E hooks

When `e2eHooksEnabled()` (local dev + auth emulator + `VITE_E2E_HOOKS !== false`):

- Auto sign-in, auto-open Studio
- `window.__TERRARIUM_E2E__.getState()` / `.waitFor(fn, ms)`
- `document.body.dataset.e2eReady === "true"` when idle

### Test IDs

| Area | IDs |
|------|-----|
| HUD | `e2e-hud-studio`, `e2e-hud-faucet`, `e2e-hud-sign-in`, `e2e-hud-jump` |
| Studio | `e2e-studio-test`, `e2e-studio-play`, `e2e-studio-deploy`, `e2e-studio-close` |
| Deploy | `e2e-deploy-confirm`, `e2e-deploy-extra`, `e2e-deploy-location` |
| World | `e2e-world-map` |

Targets: `HudOverlay.tsx`, `CreatureStudio.tsx`, `WorldCanvas.tsx`.

---

## API smoke

Script: [`scripts/api-smoke.sh`](../../scripts/api-smoke.sh) — health → auth → compile → sandbox → deploy.

```bash
SMOKE_DEPLOY=0 npm run smoke
SMOKE_DEPLOY_X=40 SMOKE_DEPLOY_Y=40 npm run smoke
```

---

## Scenarios

YAML in [scenarios/](scenarios/) — Playwright runs all via `e2e/scenarios.spec.ts`.

| Scenario | File |
|----------|------|
| Compile + sandbox | `studio-compile-test.yaml` |
| Deploy creature | `deploy-creature.yaml` |
| Auth gate | `signed-out-gate.yaml` |

Schema: [scenarios/README.md](scenarios/README.md).

---

## Browser QA (Cursor)

Skill: [`.cursor/skills/browser-qa/SKILL.md`](../../.cursor/skills/browser-qa/SKILL.md)

Pitfalls: Studio closed → `pointer-events: none`; pick map cell before deploy dialog; fresh screenshot before xy clicks.

---

## npm scripts

| Script | Description |
|--------|-------------|
| `npm run smoke` | API smoke |
| `npm run preflight` | Stack health before browser QA |
| `npm run e2e` | Preflight + Playwright |
| `npm run test:integration` | smoke then e2e |

CI: `test / smoke` + `test / e2e` (required on `main`). Detail: [../workflow/ci.md](../workflow/ci.md).
