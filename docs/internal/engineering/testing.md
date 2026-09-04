# Testing

Full dev/QA loop: [../workflow/README.md](../workflow/README.md). Doc map: [../../README.md](../../README.md).

## Quick check

```bash
./scripts/test.sh
```

Same as the reusable CI test job (minus Docker): openapi sync check, `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`, frontend `npm run lint`, release build.

OpenAPI: edit `crates/server/src/openapi.json`, then `./scripts/sync-openapi.sh` for Mintlify.

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
| **Sim unit** | Assembler bugs, VM semantics, sandbox replay | `cargo test -p terrarium-sim` |
| **Server sim** | DB persistence, tick loop, UNIQUE position updates | `cargo test -p terrarium-server` |
| **Manual dev** | Auth, Studio, deploy flow | `./scripts/dev.sh` → http://localhost:5173 |
| **QA (local)** | API smoke + Playwright e2e | `npm run test:integration` (requires dev stack) |
| **E2E (CI)** | Full stack in GitHub Actions | `./scripts/ci-e2e.sh` via `reusable-test.yml` `e2e` job |

## Sim tests

- VM and sandbox tests use minimal inline WAT in test modules — no bundled example programs.
- Default Studio source is `apps/skin/src/lib/creatureEditor.ts` (`DEFAULT_RUST_SOURCE`) — compile path tested by `npm run smoke` / Playwright.
- VM tests run creatures for N ticks in memory — no DB, fast.

Gas and action energy constants: [sim/host-abi.md](sim/host-abi.md) § Gas & energy costs.

## Server sim tests

`crates/server/src/engine.rs` uses in-memory SQLite (tests only).

Tests call `WorldEngine::tick_step()` directly — same sim path as the live 2 Hz loop.

## Local dev gotcha

`scripts/dev.sh` uses `sqlite://data/terrarium.db?mode=rwc`, not `sqlite::memory:`.

Plain `:memory:` gives **each pool connection its own empty database**, so deploy, ticks, and API reads could disagree. Use a file DB locally; reset with `rm data/terrarium.db` if corrupted (see [../workflow/troubleshooting.md](../workflow/troubleshooting.md)).

## Manual smoke test

1. `./scripts/dev.sh`
2. Open http://localhost:5173 (auto sign-in in QA mode)
3. Studio → **Test** → **Play** (sandbox moves)
4. Pick map cell → **Deploy** → confirm (~110 glims; use faucet if needed)
5. `curl -s localhost:8080/api/v1/world | jq '.creatures | length'` — count increases

Or run `npm run test:integration` for automated smoke + e2e.

## CI

| Workflow | Trigger | Checks |
|----------|---------|--------|
| `ci.yml` | PR | Parallel: Rust (fmt, clippy, test, build), frontend (lint, build), Docker build, **QA (API smoke + Playwright e2e)** |
| `deploy.yml` | push to `main`, manual | Same test jobs → parallel Cloud Run + Hosting deploy → post-deploy smoke |

Both use `.github/workflows/reusable-test.yml`. Push to `main` does not duplicate the test job across two workflows.

See [../workflow/README.md](../workflow/README.md) and [../qa/README.md](../qa/README.md) for env vars, QA hooks, and troubleshooting.
