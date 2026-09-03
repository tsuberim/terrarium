# Secrets

**Scope:** env var names and GitHub secret names. Not values — never commit those.

Used by: [deploy.md](deploy.md), [../workflow/setup.md](../workflow/setup.md).

---

## Local (`.env`, gitignored)

| Variable | Purpose |
|----------|---------|
| `FIREBASE_PROJECT_ID` | Firebase / GCP project |
| `LISTEN_ADDR` | API bind, default `0.0.0.0:8080` |
| `DATABASE_URL` | Default `sqlite://data/terrarium.db?mode=rwc` |
| `FIREBASE_AUTH_EMULATOR_HOST` | `127.0.0.1:9099` when using emulator |
| `COMPILE_WORKER_URL` | `http://127.0.0.1:8081` local worker |
| `FAUCET_ENABLED` | `true` / `false` |
| `FAUCET_MAX` | Max credits per faucet request |
| `GCP_PROJECT_ID` | GCP project for deploy |
| `GCP_REGION` | e.g. `us-central1` |
| `ARTIFACT_REPO` | Default `terrarium` |
| `CLOUD_RUN_SERVICE` | Default `terrarium-server` |
| `COMPILE_RUN_SERVICE` | Default `terrarium-compile` |

Frontend: `apps/skin/.env.local` from `setup-dev.sh`. Keys: `VITE_USE_AUTH_EMULATOR`, `VITE_QA_MODE` — see [setup.md](../workflow/setup.md).

---

## GitHub Actions secrets

| Secret | Purpose |
|--------|---------|
| `GCP_PROJECT_ID` | GCP project |
| `GCP_REGION` | Deploy region |
| `CLOUD_RUN_SERVICE` | Cloud Run service name |
| `ARTIFACT_REPO` | Docker repo |
| `FIREBASE_PROJECT_ID` | Firebase project |
| `FIREBASE_API_KEY` | Web SDK |
| `FIREBASE_AUTH_DOMAIN` | Web SDK |
| `FIREBASE_APP_ID` | Web SDK |
| `GCP_WORKLOAD_IDENTITY_PROVIDER` | WIF provider |
| `GCP_SERVICE_ACCOUNT` | e.g. `github-deploy@PROJECT.iam.gserviceaccount.com` |
| `FIREBASE_SERVICE_ACCOUNT` | JSON for Hosting deploy |

---

## Enable deploy

Repo variable: `DEPLOY_ENABLED=true` (Settings → Actions → Variables).

Without it, deploy workflow skips Cloud Run and Hosting.
