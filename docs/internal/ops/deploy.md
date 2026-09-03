# Prod deploy

**Scope:** production architecture and deploy. Not local dev or CI job detail.

Local dev: [../workflow/setup.md](../workflow/setup.md). CI: [../workflow/ci.md](../workflow/ci.md). Bootstrap: [environments.md](environments.md). Secrets: [secrets.md](secrets.md).

**Prod URL:** https://terrarium-506917.web.app · **GCP:** `terrarium-506917` (`us-central1`)

---

## Architecture

```
Browser → Firebase Hosting (SPA)
       → /api/** → Cloud Run terrarium-server
       → WebSocket → Cloud Run direct (VITE_WS_BASE)
```

| Layer | Prod |
|-------|------|
| Frontend | Firebase Hosting |
| API + sim | Cloud Run `terrarium-server` |
| Compile | Cloud Run `terrarium-compile` (optional) |
| Auth | Firebase Auth |
| Database | Ephemeral SQLite ([tech-debt](../engineering/tech-debt.md) TD-INF-1) |

---

## Build requirements

- `VITE_API_BASE=""` — REST via Hosting rewrite
- `VITE_WS_BASE=wss://<cloud-run-host>/api` — WS direct to Cloud Run

`scripts/generate-config.sh` writes `apps/skin/.env.production`. Build fails if `VITE_WS_BASE` missing.

---

## Manual deploy

```bash
./scripts/deploy-server.sh
./scripts/generate-config.sh
npm ci && npm run build --prefix apps/skin
firebase deploy --only hosting
```

Optional: `./scripts/deploy-compile-worker.sh` — set `COMPILE_WORKER_URL` on API.

Requires `gcloud`, `docker`, `.env` with `GCP_*` + `FIREBASE_PROJECT_ID`.

---

## Cost (ballpark)

| Setting | Monthly |
|---------|---------|
| Cloud Run `min-instances=0` | ~$0–5 |
| Firebase Hosting + Auth | ~$0 hobby |
| Artifact Registry | <$1 |

---

## Prod scripts

| Script | Use |
|--------|-----|
| `deploy-server.sh` | Cloud Run API |
| `deploy-compile-worker.sh` | Compile worker |
| `generate-config.sh` | `VITE_WS_BASE` for build |
| `smoke-prod.sh` | Post-deploy health |

---

## Prod troubleshooting

| Symptom | Fix |
|---------|-----|
| `/api/health` 404 on `.web.app` | Deploy Cloud Run |
| WebSocket fails | Rebuild with `generate-config.sh`, redeploy hosting |
| World empty after deploy | Ephemeral DB — expected until persistent storage |
| Cold start timeout | Wait ~45s or raise min-instances |
| Studio compile fails | Deploy compile worker; wire `COMPILE_WORKER_URL` |

Local issues → [../workflow/troubleshooting.md](../workflow/troubleshooting.md).
