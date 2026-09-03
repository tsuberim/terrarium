# Terrarium Rust SDK

Write creatures in **Rust**, compile via Creature Studio or the isolated compile worker, test in sandbox, deploy WASM.

Player docs: [Rust SDK on Mintlify](https://terrarium.mintlify.app/reference/rust-sdk).

## In-game (recommended)

1. Sign in → open **Studio** from the HUD
2. Edit body-only Rust above the `---` line; `#[terrarium::scenario]` blocks go below
3. **Test** — compiles via worker, runs sandbox scenarios
4. Pick map cell → **Deploy** — costs glims; immutable on the live world

## Local compile worker

```bash
./scripts/run-compile-worker.sh   # port 8081
COMPILE_WORKER_URL=http://127.0.0.1:8081 ./scripts/dev.sh
```

## CLI test (compile + sandbox)

```bash
export TERRARIUM_API_BASE=http://localhost:8080/api
export TERRARIUM_API_KEY=tr_…   # or FIREBASE_ID_TOKEN
./scripts/sim.sh path/to/user.rs
```

## Layout

| Path | Role |
|------|------|
| `terrarium-sdk/` | Host imports + helpers (`prelude`) |
| `scripts/sim.sh` | Compile + sandbox via API |

Player code is **body-only** above `---`; scenario attrs below are for sandbox only, not compiled into the deploy artifact.
