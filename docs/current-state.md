# Current state

Honest snapshot as of 2026-08-28.

## Shipped on main

| Layer | What runs |
| --- | --- |
| **Host** (`crates/host`) | Always-on native `World` (800k torus), ~20 Hz tick loop, WebSocket `/ws`, internal spawn/snapshot API for the credits rail |
| **Skin** (`apps/skin`) | Firebase Hosting — WebSocket camera only (480px short axis, edge-to-edge cover). No WASM tick in tab. |
| **Site** (`apps/site`) | Landing + about on Firebase Hosting |
| **Dashboard** (`apps/dashboard`) | Console on Firebase Hosting — Firebase Auth + `/dashboard/api/*` |
| **API** (`crates/api`) | Credits ledger, scoped API tokens, `POST /v1/spawn`. Delegates to host when `TERRARIUM_HOST_URL` set; in-process world for CI/local |
| **Kernel** (`crates/kernel`) | Auth-free physics + mass ledger |

## Hosting

- **Firebase Hosting** — site, skin, dashboard. Deploy workflows patch meta tags then `firebase deploy`.
- **Cloud Run** — host (`min-instances=1`, `max-instances=1`) and API in `us-central1`. GCS skin deploy retired.
- **Secrets** — `TERRARIUM_HOST_TOKEN` for API ↔ host internal routes (GitHub secret).

## Local development

```bash
./scripts/run-host.sh          # :8080 — world + WebSocket (+ skin when SKIN_DIR set)
./scripts/run-api.sh           # :3000 — credits/tokens/spawn
```

```bash
export TERRARIUM_HOST_URL=http://127.0.0.1:8080
export TERRARIUM_HOST_TOKEN=dev-shared-secret   # optional locally
```

## CI

`cargo test` for kernel, api, and host; required docs check.

## Not yet

- Live Stripe checkout
- Cash-out verb, attach/split, WASM guest modules inside cells
