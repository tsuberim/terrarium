# Current state

Honest snapshot as of 2026-08-28.

**Playable local world + platform v1.** The kernel ticks a deterministic fixed-point 2D toroidal `World`; the skin loads that same crate as WASM in the browser tab and lets you paste a bytecode program onto a cell. Credits, API tokens, and spawn rail live in `crates/api`. There is no always-on game server wired to production yet — a separate effort handles native host + Firebase Hosting deploy.

What exists:

- Docs in `/docs` — vision, architecture, **product-specs** (source of truth for UX).
- `crates/kernel` — mass ledger, toroidal wrap (`WORLD_WIDTH` / `WORLD_HEIGHT` = 800k), guest ISA, invariant tests.
- Kernel builds to WASM (`scripts/build-wasm.sh` → `apps/skin/pkg/`). Public JS surface is `JsWorld`.
- `apps/skin` — fullscreen 480px-short-axis retro camera, hideable program overlay, cross-links to home/console via meta tags. Still deployed to GCS buckets for staging/prod play URLs.
- `apps/site` — landing + about (minimal copy, shared nav). Firebase Hosting target `site`.
- `apps/dashboard` — console: Firebase Auth, credits, API tokens, billing stub. Polished to match site shell. Firebase Hosting target `dashboard`.
- `apps/shared` — shell CSS + link resolver copied into site/dashboard deploy folders.
- `crates/api` — Axum API: credits ledger, scoped tokens, `/v1/spawn`, dashboard routes. SQLite.
- `firebase.json` / `.firebaserc` — three hosting targets: site, skin, dashboard.
- CI runs `cargo test` on kernel + api and checks required docs exist.

What this is not yet: live multiplayer on the always-on host in prod, Stripe checkout, cash-out, attach/split, or retired GCS skin deploy. Guests are bytecode interpreted by the kernel.
