# Current state

Honest snapshot as of 2026-08-28 (night).

**Persistent native host.** The sim runs as a process that owns `World` and ticks without a browser. The skin is a WebSocket client (camera + program editor). Staging/prod target Cloud Run, not GCS.

What exists:

- Docs in `/docs` hold vision and architecture.
- `crates/kernel` — mass ledger, fixed-point physics, tick, tiny guest ISA. Conservation tests pass. Optional `--features wasm` for WASM builds (not the live host).
- `crates/host` — native binary: tick loop, static skin, WebSocket snapshots + `set_program` / `reset`.
- `apps/skin` — fullscreen pixel camera over WS; hideable program overlay (wander / chase / sit). Does not call `tick`.
- Dockerfile + Cloud Run deploy workflows for `terrarium-staging` / `terrarium-prod` (`us-central1`, min 1, max 1, 128Mi, CPU always allocated).
- CI: `cargo test -p terrarium-kernel`, `cargo build -p terrarium-host`, required docs present.

What this is not: multiplayer (one World per service instance), WASM guest modules inside cells, cash rail / real money, attach/split. Guests are bytecode interpreted by the kernel. Cash-out is still a later verb.
