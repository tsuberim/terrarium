# Journey

Development log. Newest last. Dates are local to the operator (Asia/Jerusalem).

## 2026-08-28

Vision locked. The dish, the closed box, mass-as-money, sleep-is-free, absorb-is-a-verb, kernel vs skin, metric-agnostic — written down in `/docs` so an outsider can read them without the chat.

Repo scaffolded: `crates/kernel` (mass ledger + conservation tests), `apps/skin` (static camera shell), GitHub Actions for `cargo test` and docs presence. Staging and prod are public Cloud Storage buckets, not a compute fleet. Infra milestone in progress: buckets, WIF, and a `gcloud storage cp` deploy still to be wired on the operator side. No simulation yet.

## 2026-08-28 (playable dish)

First playable milestone. Kernel grew a deterministic fixed-point dish, a tick, and verbs wired through a tiny bytecode ISA (`thrust` / `sense` / `absorb` / `dump`; sleep free). Same crate compiles to WASM; `apps/skin` loads it, draws the dish, shows mass burning, and ships wander / chase / sit demos plus a paste-a-program editor. Still static-only — no new cloud services. Real-money economy and WASM-in-WASM guests deferred.

## 2026-08-28 (fullscreen retro skin)

Skin UX pass: world fills the viewport as a chunky pixel framebuffer (`image-rendering: pixelated`), CRT scanlines, almost no chrome. HUD/stats (tick, mass totals, house burned) removed — raw sim only. Program editor kept as a hideable overlay so writing a creature program does not break the fullscreen feel. Docs updated. Kernel untouched.

## 2026-08-28 (native host)

Live sim leaves the browser tab. `crates/host` owns one `World`, ticks without a client open, serves the skin, and pushes state over WebSocket. Skin is camera + program editor only. GCS `gcloud storage cp` deploy workflows replaced with Cloud Run (`terrarium-staging` / `terrarium-prod`, min 1, 128Mi, CPU always on). Docs (`architecture`, `current-state`, `environments`) updated. WASM build remains optional for tests / later guests.
