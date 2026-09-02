# Testing

## Quick check

```bash
./scripts/test.sh
```

Same as the reusable CI test job (minus Docker): `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`, frontend `npm run lint`, release build.

### Pre-commit

```bash
pip install pre-commit   # or: brew install pre-commit
pre-commit install       # also run from ./scripts/setup-dev.sh when pre-commit is on PATH
pre-commit run --all-files
```

Hooks: `cargo fmt`, `cargo clippy -D warnings`, `eslint` (skin `src/` only).

## Layers

| Layer | What it catches | Command |
|-------|-----------------|---------|
| **Sim unit** | Assembler bugs, VM semantics, example programs | `cargo test -p terrarium-sim` |
| **Server sim** | DB persistence, tick loop, UNIQUE position updates | `cargo test -p terrarium-server` |
| **Manual dev** | Auth, UI, deploy flow | `./scripts/dev.sh` → http://localhost:5173 |

## Sim tests

- Every example program in `crates/sim/src/examples.rs` must compile as WAT (mirrors `apps/skin/src/lib/examples.ts`).
- Predator/scavenger examples are authored in Rust under `strategies/` and synced via `./scripts/build-strategies.sh` (dev-only; output is committed WAT).
- VM tests run creatures for N ticks in memory — no DB, fast.

## Logic tests

Sim semantic tests live in `crates/sim/src/logic_tests.rs`. They cover:

- Stack underflow stalls (does not kill)
- `eq` / `sub` pop order
- Blocked move still costs energy
- Example program behavior (wall north)
- `sense` sees creatures
- Energy zero → death
- One action per tick; rotate-then-eat ordering; frontal vision cone

Run: `cargo test -p terrarium-sim logic_tests`

When fixing a logic bug, add a test there first.

```rust
#[test]
fn my_program_does_x() {
    let mut creatures = vec![/* ... */];
    run_tick(&mut creatures, &mut HashMap::new());
    assert_eq!(creatures[0].x, expected);
}
```

## Server sim tests

`crates/server/src/engine.rs` uses in-memory SQLite.

Tests call `WorldEngine::tick_step()` directly — same sim path as the live 2 Hz loop.

## Local dev gotcha (fixed)

`scripts/dev.sh` uses `sqlite:data/terrarium.db`, not `sqlite::memory:`.

Plain `:memory:` gives **each pool connection its own empty database**, so deploy, ticks, and API reads could disagree. Use a file DB locally, or `max_connections(1)` with shared memory for tests only.

## Manual smoke test

1. `./scripts/dev.sh`
2. Sign in, faucet, deploy **Tunnel east** on an empty cell
3. Within ~1s the creature should move east (world polls every 1s)
4. `curl -s localhost:8080/api/v1/world | jq '.creatures[].x'` — `x` should increase over time

## CI

| Workflow | Trigger | Checks |
|----------|---------|--------|
| `ci.yml` | PR | Parallel: Rust (fmt, clippy, test, build), frontend (lint, build), Docker build |
| `deploy.yml` | push to `main`, manual | Same test jobs → parallel Cloud Run + Hosting deploy → post-deploy smoke |

Both use `.github/workflows/reusable-test.yml` so PR and prod paths run identical checks. Push to `main` does not duplicate the test job across two workflows.
