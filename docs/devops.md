# DevOps

How Terrarium is built, run locally, and deployed to production.

## Architecture

```
Browser
  │
  ├─ prod ──► Firebase Hosting (static SPA)
  │              │
  │              └─ /api/** ──► Cloud Run (terrarium-server)
  │
  └─ local ──► Vite :5173
                  │
                  └─ /api ──proxy──► Rust API :8080
```

| Layer | Tech | Notes |
|-------|------|-------|
| Frontend | Vite + React (`apps/skin/`) | Dark UI, Firebase Auth client |
| API | Rust / Axum (`crates/server/`) | JWT verification, credits faucet |
| Auth | Firebase Auth | Same GCP/Firebase project |
| Database | SQLite | In-memory in prod & default local dev |

**Prod URL:** https://terrarium-506917.web.app  
**GCP project:** `terrarium-506917` (region `us-central1`)

The server registers routes twice: bare paths (`/health`, `/v1/*`) for local dev, and under `/api/*` for Firebase Hosting → Cloud Run rewrites.

---

## Local development

### One-time setup

```bash
chmod +x scripts/*.sh
./scripts/setup-dev.sh
```

This creates `.env`, fetches Firebase web config into `apps/skin/.env.local`, installs npm deps, and installs `cargo-watch`.

**Firebase auth locally:** add `localhost` to [Authorized domains](https://console.firebase.google.com/project/terrarium-506917/authentication/settings) and enable Google under Sign-in method.

### Start / stop

| Script | Purpose |
|--------|---------|
| `./scripts/dev-bg.sh` | API + Vite in background (watch mode) |
| `./scripts/dev.sh` | Foreground watch — preferred for long sessions |
| `./scripts/dev-stop.sh` | Kill processes on `:8080` and `:5173` |
| `./scripts/dev-status.sh` | Quick health check |
| `./scripts/run-server.sh` | API only (no watch) |

Open **http://127.0.0.1:5173**. Vite proxies `/api` to the API on `:8080`.

Logs (background mode): `.dev/logs/{api,web}.log`

### Environment defaults

| Variable | Local default | Purpose |
|----------|---------------|---------|
| `LISTEN_ADDR` | `0.0.0.0:8080` | API bind address |
| `DATABASE_URL` | `sqlite::memory:` | Avoids file-lock issues on macOS dev |
| `FAUCET_ENABLED` | `true` | Dev credits faucet |
| `FIREBASE_PROJECT_ID` | from `.env` | JWT issuer |

Run frontend or API separately if needed:

```bash
./scripts/run-server.sh
cd apps/skin && npm run dev
```

---

## Production

### Components

| Component | Platform | Details |
|-----------|----------|---------|
| API | Cloud Run `terrarium-server` | `min-instances=1`, port 8080 |
| UI | Firebase Hosting | Serves `apps/skin/dist` |
| Container registry | Artifact Registry `terrarium` | `us-central1-docker.pkg.dev/.../server` |

`firebase.json` rewrites `/api/**` to the Cloud Run service. The SPA is built with `VITE_API_BASE=""` so the browser calls same-origin `/api/...`.

### Manual deploy

Requires `gcloud`, `docker`, and `.env` with `GCP_*` + `FIREBASE_PROJECT_ID`.

```bash
./scripts/deploy-server.sh          # build linux/amd64, push, deploy Cloud Run
firebase deploy --only hosting      # after npm run build in apps/skin
```

CI does both on push to `main` (see below).

### Persistence caveat

Cloud Run is currently deployed with `DATABASE_URL=sqlite::memory:`. World state and accounts **do not survive** container restarts or redeploys. Persistent storage is a future change (Cloud SQL, attached volume, etc.).

---

## CI/CD

GitHub Actions (`.github/workflows/`):

| Workflow | Trigger | Jobs |
|----------|---------|------|
| `ci.yml` | PR + push to `main` | `cargo test`, release build, frontend build, `docker build` |
| `deploy.yml` | push to `main` | test → Cloud Run deploy → Firebase Hosting deploy |

Deploy uses Workload Identity Federation (no long-lived GCP keys in CI). Secrets are listed in [secrets.md](secrets.md).

Deploy concurrency group `deploy-prod` cancels in-progress deploys when new commits land on `main`.

---

## Scripts reference

| Script | Use |
|--------|-----|
| `setup-dev.sh` | One-time local bootstrap |
| `dev-bg.sh` / `dev.sh` / `dev-stop.sh` / `dev-status.sh` | Local dev lifecycle |
| `run-server.sh` | API without file watch |
| `deploy-server.sh` | Cloud Run deploy (local or CI) |
| `generate-config.sh` | Write `apps/skin/.env.production` for CI builds |

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| Connection refused on `:5173` | Vite not running | `./scripts/dev-bg.sh` or `./scripts/dev.sh` from your terminal |
| Connection refused on `:8080` | API not up yet | Wait for cargo build; check `.dev/logs/api.log` |
| Sign-in fails locally | Firebase config | Enable Google provider; add `localhost` to authorized domains |
| `/api/health` 404 in prod | Cloud Run not deployed | Check deploy workflow or run `deploy-server.sh` |
| Data gone after deploy | In-memory SQLite | Expected until persistent DB is configured |
