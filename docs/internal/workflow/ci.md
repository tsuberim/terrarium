# CI

**Scope:** GitHub Actions and local CI parity. Not local dev setup or prod deploy steps.

Local fast check: `./scripts/test.sh`. Full stack: `npm run test:integration`.

---

## Workflows

| Workflow | Trigger | Jobs |
|----------|---------|------|
| `ci.yml` | PR | `reusable-test.yml` |
| `deploy.yml` | push `main`, manual | test → deploy → smoke |

---

## `reusable-test.yml` jobs

| Job | Checks |
|-----|--------|
| **rust** | fmt, clippy, test, release build |
| **frontend** | lint, build |
| **docker** | image build (no push) |
| **smoke** | `./scripts/ci-api-smoke.sh` — API only, no Vite |
| **e2e** | `./scripts/ci-e2e.sh` — Playwright UI scenarios |
| **gate** | fail if any job failed |

Path filters skip irrelevant jobs.

**Required checks on `main`:** `test / rust`, `test / frontend`, `test / docker`, `test / smoke`, `test / e2e`.

---

## CI scripts

| Script | Stack | Runs |
|--------|-------|------|
| `ci-stack.sh` | shared bootstrap (sourced) | — |
| `ci-api-smoke.sh` | auth + API + compile worker | `api-smoke.sh` |
| `ci-e2e.sh` | full stack + Vite | Playwright |

DB: `data/terrarium-ci.db`.

---

## Local integration tests

| Command | What |
|---------|------|
| `npm run smoke` | API smoke (curl) |
| `npm run e2e` | Preflight + Playwright |
| `npm run test:integration` | smoke then e2e |
| `npm run preflight` | Health-check dev stack |

Smoke options: `SMOKE_DEPLOY=0 npm run smoke`

---

## `./scripts/test.sh`

Same as CI rust + frontend (no Docker):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release -p terrarium-server
cd apps/skin && npm ci && npm run lint && npm run build
```

---

## Pre-commit

```bash
pip install pre-commit && pre-commit install
pre-commit run --all-files
```

Hooks: `cargo fmt`, `cargo clippy -D warnings`, eslint (skin `src/`).

---

## Deploy (main)

Requires secrets ([../ops/secrets.md](../ops/secrets.md)) and `DEPLOY_ENABLED=true`.

Post-deploy: `scripts/smoke-prod.sh`. Detail: [../ops/deploy.md](../ops/deploy.md).
