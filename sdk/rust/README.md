# Terrarium Rust SDK

Write creatures in **Rust**, compile via the in-game editor or isolated compile worker, test in sandbox, deploy WASM.

## In-game (recommended)

1. Click an empty cell → deploy dialog
2. Edit `user.rs` body in the Monaco editor
3. **Test** — compiles via Cloud Run worker, previews in sandbox scenarios
4. **Deploy** — costs glims, immutable on the live world

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

Player code implements `pub fn tick()` in the locked template (`services/compile-worker/template/`).
