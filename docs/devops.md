# DevOps

How Terrarium is built, run locally, and deployed to production.

## Architecture

```
Browser
  │
  ├─ prod ──► Firebase Hosting (static SPA)
  │              ├─ HTTP /api/** ──► Cloud Run (rewrite)
  │              └─ WebSocket ──► Cloud Run direct (VITE_WS_BASE, not Hosting)
  │
  └─ local ──► Vite :5173 (self-contained, no Cloud Run)
                  └─ /api + WS ──proxy──► Rust API :8080
```

| Layer | Tech | Notes |
|-------|------|-------|
| Frontend | Vite + React (`apps/skin/`) | Dark UI, Firebase Auth client |
| API | Rust / Axum (`crates/server/`) | JWT or API keys, OpenAPI at `/api/docs` |
| Auth | Firebase Auth | Same GCP/Firebase project |
| Database | SQLite | Cloud Run: `sqlite:/tmp/terrarium.db` (ephemeral); local: `data/terrarium.db` |

**Prod URL:** https://terrarium-506917.web.app  
**GCP project:** `terrarium-506917` (region `us-central1`)

The server registers routes twice: bare paths (`/health`, `/v1/*`) for local dev, and under `/api/*` for Firebase Hosting → Cloud Run rewrites.

---

## Local development (self-contained)

Local dev does **not** use Cloud Run, prod WebSocket URLs, or GCP deploy credentials. Everything runs on your machine:

| Piece | Where |
|-------|--------|
| UI | Vite `:5173` |
| API + sim | Rust server `:8080` (via `cargo watch`) |
| HTTP | Browser → `/api/...` → Vite proxy → `:8080` |
| WebSocket | Browser → `ws://127.0.0.1:8080/api/...` (not through Hosting) |
| Database | `data/terrarium.db` (persistent across restarts) |
| Auth | Firebase (same project as prod, for sign-in only) |

```bash
./scripts/setup-dev.sh   # once — writes apps/skin/.env.local (VITE_WS_BASE empty)
./scripts/dev.sh         # starts API + Vite together
```

Open **http://localhost:5173**.

**Do not** open the prod URL (`terrarium-506917.web.app`) when developing — that hits deployed Cloud Run, which may lag behind your local code.

**Do not** set `VITE_WS_BASE` in `.env.local` unless you intentionally want the dev UI to talk to prod Cloud Run.

**API docs:** http://localhost:5173/api/docs · **OpenAPI:** http://localhost:5173/api/openapi.json

Stop: Ctrl+C or `./scripts/dev-stop.sh`.

---

## Production

### Components

| Component | Platform | Details |
|-----------|----------|---------|
| API | Cloud Run `terrarium-server` | `min-instances=0` default; use On/Off or `server-power.sh` |
| UI | Firebase Hosting | Serves `apps/skin/dist` |
| Container registry | Artifact Registry `terrarium` | `us-central1-docker.pkg.dev/.../server` |

`firebase.json` rewrites `/api/**` to Cloud Run for **HTTP only**. The SPA is built with:

- `VITE_API_BASE=""` — same-origin `/api/...` for REST (deploy, /me, /world)
- `VITE_WS_BASE=wss://<cloud-run-host>/api` — WebSocket **must** connect directly to Cloud Run; Firebase Hosting cannot proxy WS upgrades

CI and manual prod builds run `scripts/generate-config.sh`, which fetches the Cloud Run URL and writes `apps/skin/.env.production`. Builds fail if `VITE_WS_BASE` is missing.

### Manual deploy

Requires `gcloud`, `docker`, and `.env` with `GCP_*` + `FIREBASE_PROJECT_ID`.

```bash
./scripts/deploy-server.sh          # build linux/amd64, push, deploy Cloud Run
./scripts/generate-config.sh        # writes .env.production with TERRARIUM_WS_BASE from Cloud Run
npm ci && npm run build --prefix apps/skin
firebase deploy --only hosting
```

CI does both on push to `main` (see below).

### Cost (ballpark)

| Setting | Typical monthly cost |
|---------|---------------------|
| **`min-instances=1`** (old default) | ~**$15–40** — one always-warm Cloud Run instance (512Mi–1CPU) |
| **`min-instances=0`** (new default) | ~**$0–5** — pay only when someone hits the API; cold start ~15–45s |
| Firebase Hosting + Auth | Usually **$0** at hobby traffic |
| Artifact Registry | **&lt;$1** |

Deploy now sets **`MIN_INSTANCES=0`**. Use **Wake server** in the HUD when offline, or `./scripts/server-power.sh on` to keep it warm.

### Server on/off

**UI (admin):** Set `ADMIN_UIDS` to your Firebase uid(s) on Cloud Run. Grant the **runtime service account** `roles/run.admin` so the API can patch its own scaling. Signed-in admins see **On / Off** in the left HUD panel.

**CLI:**

```bash
./scripts/server-power.sh off   # min-instances=0 — save money
./scripts/server-power.sh on    # min-instances=1 — always warm
./scripts/server-power.sh status
```

**Sleep note:** **Off** scales to zero after the request finishes; the world pauses until someone **Wake**s (anyone) or you run `server-power.sh on`.

### Persistence

Cloud Run uses `DATABASE_URL=sqlite:///app/data/terrarium.db` (ephemeral per instance; `/app/data` exists in the image).

---

## CI/CD

GitHub Actions (`.github/workflows/`):

| Workflow | Trigger | Jobs |
|----------|---------|------|
| `ci.yml` | PR + push to `main` | `cargo test`, release build, frontend build, `docker build` |
| `deploy.yml` | push to `main` | test → Cloud Run + Firebase Hosting (parallel after test) |

Deploy uses Workload Identity Federation (no long-lived GCP keys in CI). Secrets are listed in [secrets.md](secrets.md).

Deploy concurrency group `deploy-prod` cancels in-progress deploys when new commits land on `main`.

---

## Scripts reference

| Script | Use |
|--------|-----|
| `setup-dev.sh` | One-time local bootstrap |
| `dev.sh` / `dev-stop.sh` | Start / stop local dev |
| `deploy-server.sh` | Cloud Run deploy (local or CI) |
| `server-power.sh` | CLI wake/sleep (`min-instances` 0 ↔ 1) |
| `generate-config.sh` | Write `apps/skin/.env.production` for CI builds |

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| Connection refused on `:5173` | Dev not running | `./scripts/dev.sh` in a terminal tab |
| Sign-in fails locally | Firebase config | Enable Google provider; add `localhost` to authorized domains |
| `/api/health` 404 in prod | Cloud Run not deployed | Check deploy workflow or run `deploy-server.sh` |
| WebSocket fails on `.web.app` | Prod build missing `VITE_WS_BASE` (falls back to Hosting origin) | Redeploy frontend via CI or run `generate-config.sh` + `npm run build` + `firebase deploy --only hosting` |
| Data gone after deploy | In-memory SQLite | Expected until persistent DB is configured |
