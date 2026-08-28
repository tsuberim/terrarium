# Journey

Development log. Newest last. Dates are local to the operator (Asia/Jerusalem).

## 2026-08-28

Vision locked. The world, the closed box, mass-as-money, sleep-is-free, absorb-is-a-verb, kernel vs skin, metric-agnostic — written down in `/docs` so an outsider can read them without the chat.

Repo scaffolded: `crates/kernel` (mass ledger + conservation tests), `apps/skin` (static camera shell), GitHub Actions for `cargo test` and docs presence. Staging and prod are public Cloud Storage buckets, not a compute fleet. Infra milestone in progress: buckets, WIF, and a `gcloud storage cp` deploy still to be wired on the operator side. No simulation yet.

## 2026-08-28 (playable world)

First playable milestone. Kernel grew a deterministic fixed-point world, a tick, and verbs wired through a tiny bytecode ISA (`thrust` / `sense` / `absorb` / `dump`; sleep free). Same crate compiles to WASM; `apps/skin` loads it in the browser tab, draws the world, and ships wander / chase / sit demos plus a paste-a-program editor. Still static-only — no game server. Real-money economy and WASM-in-WASM guests deferred.

## 2026-08-28 (fullscreen retro skin)

Skin UX pass: world fills the viewport as a chunky pixel framebuffer (`image-rendering: pixelated`), CRT scanlines, almost no chrome. HUD/stats (tick, mass totals, house burned) removed — raw sim only. Program editor kept as a hideable overlay so writing a creature program does not break the fullscreen feel. Docs updated. Kernel untouched.

## 2026-08-28 (drop dish; ledger invariants)

Dropped "dish" terminology everywhere (code, skin, docs). Public names are `World` / `WORLD_RADIUS` / `worldRadius` / `#world`. Added kernel invariant tests (closed ledger with `spawned_mass`, monotonic `house_burned`, dump/absorb conservation, free sleep/halt/empty, tick determinism, world bounds, spend/dump-to-zero, sense cost) under the existing CI `cargo test` job. Docs note: sim runs as WASM in the browser; native tests share the crate; no game server yet. GCS hosting unchanged.
