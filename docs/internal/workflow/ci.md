# CI

**Scope:** GitHub Actions and local CI parity. Not local dev setup or prod deploy steps.

Local fast check: `./scripts/test.sh`. Full stack QA: `./scripts/ci-qa.sh`.

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
| **qa** | `./scripts/ci-qa.sh` |
| **gate** | fail if any job failed |

Path filters skip irrelevant jobs.

---

## `ci-qa.sh`

1. Build server + compile-worker (release)
2. QA `.env.local` (fake Firebase + emulator flags)
3. Start auth emulator, worker, server, Vite
4. Preflight (up to ~3 min)
5. `npm run qa` + Playwright
6. DB: `data/terrarium-ci.db`

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

---

## QA scripts

| Script | Use |
|--------|-----|
| `qa-preflight.sh` | Health-check services |
| `qa-smoke.sh` | API smoke (curl) |
| `qa-e2e.sh` | Preflight + Playwright |
| `ci-qa.sh` | Full stack for GitHub Actions |

Smoke options: `QA_DEPLOY=0 npm run qa`
