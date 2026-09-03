# QA Scenarios

Human- and machine-readable flows shared by Playwright specs and Cursor browser agents.

## File format

```yaml
id: studio-compile-test          # unique slug
description: One-line summary
requires:                        # dev services that must be up
  - auth_emulator
  - compile_worker
  - api
steps:
  - assert:
      qaState:
        signedIn: true
        studioOpen: true
  - click: e2e-studio-test
  - waitFor:
      qaState:
        testing: false
        wasmReady: true
      timeout: 30000
  - click: e2e-studio-play
  - waitFor:
      qaState:
        playback: playing
```

## Step types

| Step | Fields | Meaning |
|------|--------|---------|
| `assert` | `qaState: { ... }` | State must match now (partial match) |
| `waitFor` | `qaState: { ... }`, `timeout` | Poll until match or fail |
| `click` | testid string | Click element with `data-testid` |
| `clickMap` | `x`, `y` (optional) | Click world map; omit coords to use center of map pane |
| `faucet` | — | Click `e2e-hud-faucet` if credits insufficient |
| `note` | string | Documentation only; ignored by runners |

## QA state fields

Used in `assert` / `waitFor` (via `window.__TERRARIUM_E2E__.getState()`):

| Field | Type | Description |
|-------|------|-------------|
| `ready` | boolean | Firebase auth resolved |
| `signedIn` | boolean | User signed in |
| `studioOpen` | boolean | Creature Studio visible and interactive |
| `deployCell` | `{x,y}` \| null | Selected deploy coordinates |
| `deployDialogOpen` | boolean | Deploy modal open |
| `credits` | number \| null | User glims balance |
| `testing` | boolean | Compile/sandbox in progress |
| `wasmReady` | boolean | WASM compiled or uploaded |
| `playback` | `idle` \| `playing` \| `paused` | Sandbox replay state |
| `error` | string \| null | Last action error |

## Scenarios

| File | Purpose |
|------|---------|
| [`studio-compile-test.yaml`](studio-compile-test.yaml) | Default source → Test → Play |
| [`deploy-creature.yaml`](deploy-creature.yaml) | Map pick → deploy with min glims |
| [`signed-out-gate.yaml`](signed-out-gate.yaml) | Studio unavailable when signed out |

## Runner mapping

- **Playwright:** `apps/skin/e2e/scenarios.spec.ts` loads every `*.yaml` via `helpers/run-scenario.ts`
- **Cursor agent:** read scenario YAML, execute steps via browser MCP + QA bridge (see `.cursor/skills/browser-qa`)
- **API smoke:** backend-only subset (no `click` steps); see `scripts/api-smoke.sh`
