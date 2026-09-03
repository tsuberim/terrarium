---
name: browser-qa
description: >-
  Manual browser QA for Terrarium using the Cursor IDE browser MCP. Use when
  the user asks to QA, test, or verify the UI locally; when doing exploratory
  manual testing of Studio, deploy, or map flows; or alongside Playwright e2e for
  exploratory checks. Read docs/internal/qa/README.md and docs/internal/workflow/README.md.
---

# Terrarium Browser QA

Exploratory end-user QA in the Cursor browser tab. Complements headless `npm run qa`.

## Prerequisites

1. Dev stack running: `./scripts/dev.sh`
2. Services up:
   - http://127.0.0.1:8080/api/health
   - http://127.0.0.1:8081/health (`body_wrap: true`)
   - http://127.0.0.1:9099 (auth emulator)
   - http://localhost:5173 (Vite)

Run `npm run qa:preflight` then `npm run qa` for sanity checks.

## Start

```
browser_navigate → http://localhost:5173
```

With auth emulator enabled, the app auto-signs-in as `qa@terrarium.dev` and opens Studio.

Verify state:

```js
window.__TERRARIUM_QA__.getState()
// expect: { signedIn: true, studioOpen: true, wasmReady: false, ... }

document.body.dataset.qaReady === "true"  // idle + ready
```

## Core flows

Scenarios: [docs/internal/qa/scenarios/](../../docs/internal/qa/scenarios/).

### Studio compile + test

1. Click **Studio** (`qa-hud-studio`) if studio not open
2. Click **Test** (`qa-studio-test`) — wait for "Testing…" to finish
3. Verify WASM ready (upload button shows `creature.wasm`)
4. Click **Play** (`qa-studio-play`) → **Pause** / **Stop**

### Deploy creature

**Order matters:**

1. Run Test first (need WASM)
2. **Pick map cell before opening deploy dialog** — click the world map area to the right of the studio pane
3. URL should gain `x=` and `y=` params; status bar shows coordinates
4. Click **+100 glims** if deploy dialog says insufficient glims (min cost ~110)
5. Click **Deploy** (`qa-studio-deploy`)
6. Confirm in dialog (`qa-deploy-confirm`)

### Map clicks

1. `browser_take_screenshot` first (required before xy clicks)
2. Click on the **map pane** (right side when studio is open), not the deploy modal
3. Target element: canvas / `qa-world-map`
4. If deploy dialog is open, click map areas **outside** the centered panel

## Browser MCP workflow

```
browser_navigate → http://localhost:5173
browser_snapshot                    # get element refs
browser_click(ref)                  # prefer over CDP synthetic clicks
browser_take_screenshot             # before browser_mouse_click_xy
browser_mouse_click_xy(x, y)        # map picks only
browser_cdp → Runtime.evaluate      # read __TERRARIUM_QA__ when available
```

Lock tab for long sessions: `browser_lock` → interact → `browser_unlock`.

## Pitfalls

| Issue | Cause | Fix |
|-------|-------|-----|
| Clicks on Test/Deploy do nothing | Studio shell closed (`studio-shell-closed`) | Open Studio first; closed shell has `pointer-events: none` |
| Map click selects deploy dialog | Click landed on modal | Cancel deploy; pick cell with dialog closed, or click far left/right of map |
| `browser_mouse_click_xy` fails | Stale screenshot | Take fresh screenshot first |
| CDP pointer events blocked | Cursor smart mode | Use `browser_click` / `browser_mouse_click_xy`, not raw CDP dispatch |
| Not signed in | Emulator off or fresh DB | Check `VITE_USE_AUTH_EMULATOR=true`; reload page |
| Deploy disabled | Need ~110 glims | Click +100 glims faucet (may need twice) |

## After browser QA

Run API confirmation:

```bash
npm run qa
```

## Reference

- Framework: [docs/internal/qa/README.md](../../docs/internal/qa/README.md)
- Scenarios: [docs/internal/qa/scenarios/](../../docs/internal/qa/scenarios/)
- Headless smoke: [scripts/qa-smoke.sh](../../scripts/qa-smoke.sh)
- QA user: `qa@terrarium.dev` / `qa-terrarium`

## When to use Playwright instead

Use `npm run qa:e2e` for:

- Repeatable regression runs
- Same flow every time without agent judgment
- CI (`.github/workflows/reusable-test.yml` `qa` job)

Use this browser skill for layout checks, new UX, and flows not yet in Playwright (e.g. `signed-out-gate.yaml`).
