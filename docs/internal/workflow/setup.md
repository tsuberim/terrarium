# Setup & local dev

**Scope:** first-time setup and local stack. Not CI, prod, or QA detail.

Prod deploy: [../ops/deploy.md](../ops/deploy.md). QA: [../qa/README.md](../qa/README.md).

---

## First-time setup

```bash
./scripts/setup-dev.sh
```

Creates `.env`, `apps/skin/.env.local`, installs deps, optional pre-commit.

**Requires:** Rust, Node 22+, `firebase` CLI (for SDK fetch).

---

## Daily dev

```bash
./scripts/dev.sh
```

| Process | Port |
|---------|------|
| Auth emulator | 9099 |
| Compile worker | 8081 |
| API + sim | 8080 |
| Vite UI | 5173 |

Open http://localhost:5173. Stop: Ctrl+C or `./scripts/dev-stop.sh`.

---

## Server env (set by `dev.sh`)

| Variable | Default |
|----------|---------|
| `LISTEN_ADDR` | `0.0.0.0:8080` |
| `DATABASE_URL` | `sqlite://data/terrarium.db?mode=rwc` |
| `FIREBASE_AUTH_EMULATOR_HOST` | `127.0.0.1:9099` |
| `COMPILE_WORKER_URL` | `http://127.0.0.1:8081` |
| `FAUCET_ENABLED` | `true` |

---

## Frontend env (`apps/skin/.env.local`)

| Variable | Purpose |
|----------|---------|
| `VITE_API_BASE=` | Empty → Vite proxies `/api` to `:8080` |
| `VITE_WS_BASE=` | Empty → WS direct to `:8080` |
| `VITE_USE_AUTH_EMULATOR=true` | Auth emulator |
| `VITE_E2E_HOOKS=true` | Auto sign-in, Studio, QA bridge |

---

## Auth (local)

- Auto sign-in: `qa@terrarium.dev` / `qa-terrarium`
- Emulator UI: http://127.0.0.1:4000/auth

---

## Notes

- WS in dev goes direct to `:8080` (not through Vite). Prod needs `VITE_WS_BASE` — see [../ops/deploy.md](../ops/deploy.md).
- Don't open prod URL when testing local code.
- API docs: http://localhost:5173/api/docs

---

## Dev scripts

| Script | Use |
|--------|-----|
| `run-auth-emulator.sh` | Auth emulator only |
| `run-compile-worker.sh` | Compile worker only |
