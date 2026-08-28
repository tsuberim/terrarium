# Current state

Honest snapshot as of 2026-08-28 (evening).

**Playable local dish.** The kernel ticks a deterministic fixed-point 2D dish; the skin loads that same crate as WASM and lets you paste a bytecode program onto a cell.

What exists:

- Docs in `/docs` hold vision and architecture.
- `crates/kernel` — mass ledger (spawn / spend / dump / absorb) plus dish physics, tick, and a tiny guest ISA (`thrust`, `sense`, `absorb`, `dump`, `sleep`, jumps). Conservation tests still pass; tick/spend/absorb covered.
- Kernel builds to WASM (`scripts/build-wasm.sh` → `apps/skin/pkg/`).
- `apps/skin` — fullscreen pixelated camera: low-res canvas scaled with nearest-neighbor, no stats/HUD. Program editor is a hideable overlay (wander / chase / sit demos). No Cloud Run. No Docker. No always-on server.
- CI runs `cargo test` and checks that required docs exist.
- Staging and prod remain two public GCS buckets. Deploy is still `gcloud storage cp` of `apps/skin/*` (includes `pkg/`).

What this is not: multiplayer, WASM guest modules inside cells, cash rail / real money, attach/split, or a hosted always-on sim. Guests are bytecode interpreted by the kernel. Cash-out is still a later verb.
