---
name: browser-qa
description: >-
  Manual browser QA for Terrarium using the Cursor IDE browser MCP. Use when
  the user asks to QA, test, or verify the UI locally. Read docs/internal/qa/README.md.
---

# Terrarium Browser QA

Exploratory QA in the Cursor browser tab. Full framework: [docs/internal/qa/README.md](../../docs/internal/qa/README.md).

## Prerequisites

1. `./scripts/dev.sh` running
2. `npm run preflight` passes

## Start

```
browser_navigate → http://localhost:5173
```

Verify: `window.__TERRARIUM_E2E__.getState()` → `{ signedIn: true, studioOpen: true, ... }`

## Core flows

Scenarios: [docs/internal/qa/scenarios/](../../docs/internal/qa/scenarios/)

1. **Studio** — Test (`e2e-studio-test`) → Play → Stop
2. **Deploy** — pick map cell first → Deploy (`e2e-studio-deploy`) → confirm (`e2e-deploy-confirm`)

Use `browser_snapshot` + `browser_click` (prefer over CDP). Screenshot before `browser_mouse_click_xy`.

## Pitfalls

| Issue | Fix |
|-------|-----|
| Clicks do nothing | Open Studio first (`e2e-hud-studio`) |
| Map click hits modal | Pick cell with deploy dialog closed |
| Not signed in | Check auth emulator + reload |

After browser QA: `npm run smoke`.

For regression/CI use `npm run e2e` instead.
