# Current state

Honest snapshot as of 2026-08-28.

**Playable local world.** The kernel ticks a deterministic fixed-point 2D `World`; the skin loads that same crate as WASM in the browser tab and lets you paste a bytecode program onto a cell. There is no game server. `cargo test` runs the same crate natively.

What exists:

- Docs in `/docs` hold vision and architecture (source of truth).
- `crates/kernel` — mass ledger (`spawned_mass` / `total_mass` / `house_burned`) plus world physics, tick, and a tiny guest ISA (`thrust`, `sense`, `absorb`, `dump`, `sleep`, jumps). Invariant tests cover closed ledger, monotonic burn, dump/absorb conservation, free sleep/halt, tick determinism, toroidal wrap, spend/dump-to-zero, and sense cost.
- Kernel builds to WASM (`scripts/build-wasm.sh` → `apps/skin/pkg/`). Public JS surface is `JsWorld` (`worldWidth`, `worldHeight`, `tick`, `snapshot`, …).
- `apps/skin` — fullscreen pixelated camera over a wrapping rectangular world: `#world` canvas, no stats/HUD. Program editor is a hideable overlay (wander / chase / sit demos). No Cloud Run. No Docker. No always-on server.
- CI runs `cargo test --manifest-path crates/kernel/Cargo.toml` and checks that required docs exist.
- Staging and prod remain two public GCS buckets. Deploy is still `gcloud storage cp` of `apps/skin/*` (includes `pkg/`).

What this is not: multiplayer, WASM guest modules inside cells, cash rail / real money, attach/split, or a hosted always-on sim. Guests are bytecode interpreted by the kernel. Cash-out is still a later verb. Hosting stays on GCS for now.
