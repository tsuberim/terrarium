# Testing

## Quick check

```bash
./scripts/test.sh
```

Same as CI: `cargo test --workspace`, release build, frontend build.

## Layers

| Layer | What it catches | Command |
|-------|-----------------|---------|
| **Kernel unit** | Assembler bugs, VM semantics, example programs | `cargo test -p terrarium-kernel` |
| **Server sim** | DB persistence, tick loop, UNIQUE position updates | `cargo test -p terrarium-server` |
| **Manual dev** | Auth, UI, deploy flow | `./scripts/dev.sh` → http://localhost:5173 |

## Kernel tests

- Every example program in `crates/kernel/src/examples.rs` must assemble (mirrors `apps/skin/src/lib/examples.ts`).
- VM tests run creatures for N ticks in memory — no DB, fast.

## Logic tests

Kernel semantic tests live in `crates/kernel/src/logic_tests.rs`. They cover:

- Stack underflow stalls (does not kill)
- `eq` / `sub` pop order
- Blocked move still costs energy
- Example program behavior (wall north)
- `sense` sees creatures
- Energy zero → death

Run: `cargo test -p terrarium-kernel logic_tests`

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

`crates/server/src/sim.rs` uses in-memory SQLite with **one connection** (required for `:memory:`).

Tests call `tick_once` directly — same path as the live 10 Hz loop.

## Local dev gotcha (fixed)

`scripts/dev.sh` uses `sqlite:data/terrarium.db`, not `sqlite::memory:`.

Plain `:memory:` gives **each pool connection its own empty database**, so deploy, ticks, and API reads could disagree. Use a file DB locally, or `max_connections(1)` with shared memory for tests only.

## Manual smoke test

1. `./scripts/dev.sh`
2. Sign in, faucet, deploy **Tunnel east** on an empty cell
3. Within ~1s the creature should move east (world polls every 1s)
4. `curl -s localhost:8080/api/v1/world | jq '.creatures[].x'` — `x` should increase over time

## CI

`.github/workflows/ci.yml` runs `cargo test --workspace` on every push/PR.
