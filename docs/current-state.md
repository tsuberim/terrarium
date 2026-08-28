# Current state

Honest snapshot as of 2026-08-28.

**Playable local world + billing/API v1 (in progress).** The kernel ticks a deterministic fixed-point 2D `World`; the skin loads that same crate as WASM in the browser tab. A new Rust API service (`crates/api`) holds account credits, API tokens, and a native kernel world for public spawn.

What exists:

- Docs in `/docs` hold vision, architecture, and product specs (source of truth).
- `crates/kernel` — mass ledger (`spawned_mass` / `total_mass` / `house_burned`) plus world physics, tick, and a tiny guest ISA (`thrust`, `sense`, `absorb`, `dump`, `sleep`, jumps). Invariant tests cover closed ledger, monotonic burn, dump/absorb conservation, free sleep/halt, tick determinism, toroidal wrap, spend/dump-to-zero, and sense cost.
- Kernel builds to WASM (`scripts/build-wasm.sh` → `apps/skin/pkg/`). Public JS surface is `JsWorld` (`worldWidth`, `worldHeight`, `tick`, `snapshot`, …).
- `apps/skin` — fullscreen pixelated camera over a wrapping rectangular world: `#world` canvas, no stats/HUD. Program editor is a hideable overlay (wander / chase / sit demos).
- `crates/api` — Axum HTTP server: credits ledger (SQLite), API token mint/revoke, `POST /v1/spawn`, dashboard static UI at `/dashboard/`. Free-credit faucet on staging/local only.
- `apps/dashboard` — separate billing/token UI (served by the API, not skin chrome).
- CI runs `cargo test` for kernel and API, and checks required docs exist.
- Staging skin + prod skin remain public GCS buckets. API deploy is optional Cloud Run (see `docs/environments.md`); local `./scripts/run-api.sh`.

What this is not yet: live Stripe, multiplayer sync between browser skin and server world, WASM guest modules inside cells, attach/split, or cash-out. Guests are bytecode interpreted by the kernel. Cash-out is still a later verb.
