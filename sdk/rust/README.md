# Terrarium Rust SDK

Write creatures in **Rust**, compile via Creature Studio or the isolated compile worker, test in sandbox, deploy WASM.

Player docs: [Rust SDK on Mintlify](https://terrarium.mintlify.app/reference/rust-sdk).

## In-game (recommended)

1. Sign in → open **Studio** from the HUD
2. **Source** tab — creature logic (`move_forward();`, `loop { ... }`, etc.); prelude is injected at compile time
3. **Tests** tab — `#[terrarium::test]` blocks for sandbox checks
4. **Run test** → preview → pick map cell → **Deploy**

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

Player source is written to `user.rs` as-is (prelude auto-injected if missing). Tests are separate — not compiled into the deploy artifact.
