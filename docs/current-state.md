# Current state

Honest snapshot as of 2026-08-28.

## Destination (agreed in #discuss-tech)

| Layer | Target |
| --- | --- |
| Sim | Always-on **native host** (`crates/host`, PR #4) owns `World`, ticks continuously, WebSocket to browsers |
| Skin | Firebase Hosting static client — **viewer only**, no WASM tick in tab |
| Dashboard | Firebase Hosting static client — Firebase Auth, calls API for credits/tokens/billing |
| Identity | Firebase Auth (humans) + scoped API tokens (machines) |
| Economy | API credit ledger → `spawn_cell_at`; kernel stays auth-free |
| Hosting | Firebase Hosting replaces GCS for skin + dashboard |

## What exists on main today

- **`crates/kernel`** — mass ledger, toroidal world, bytecode ISA, invariant tests. Auth-free.
- **`crates/api`** — Axum service: SQLite credits, scoped API tokens (`spawn` / `read`), `POST /v1/spawn`, dashboard JSON API, staging/local free-credit faucet. Firebase JWT verify when `FIREBASE_PROJECT_ID` is set; dev session tokens when not (CI/local). Runs an in-process native `World` for spawn v1.
- **`apps/dashboard`** — static billing/token UI; Firebase Auth sign-in when configured, dev sign-in fallback.
- **`apps/skin`** — fullscreen retro camera. **Legacy:** ticks kernel WASM in the browser tab.
- **CI** — `cargo test` for kernel + API; required docs check.
- **`firebase.json`** — Hosting config for skin + dashboard (deploy workflows still copy to GCS — migration TODO).

## Legacy (still live, to be retired)

| Legacy | Replacement |
| --- | --- |
| Skin ticks WASM in-tab | Host WebSocket + skin as viewer (PR #4) |
| GCS bucket deploy (`gcloud storage cp`) | Firebase Hosting (`firebase deploy`) |
| API serves dashboard static files | Firebase Hosting serves dashboard; API is JSON only |
| Dev session tokens (`trm_sess_…`) | Firebase ID tokens in staging/prod |

## Not yet

- `crates/host` merged to main (PR #4 open)
- Live Stripe checkout
- API → host spawn delegation (API still owns world in-process for v1)
- Firebase Hosting deploy in CI (config present; workflows not switched)
- Cash-out verb, attach/split, WASM guest modules inside cells
