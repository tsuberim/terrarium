# Secrets

See [devops.md](devops.md) for where these are used.

## Local (`.env`, gitignored)

| Variable | Purpose |
|----------|---------|
| `FIREBASE_PROJECT_ID` | Firebase / GCP project id |
| `LISTEN_ADDR` | API bind, default `0.0.0.0:8080` |
| `DATABASE_URL` | Default `sqlite::memory:` for dev |
| `FAUCET_ENABLED` | `true` / `false` |
| `FAUCET_MAX` | Max credits per faucet request |
| `GCP_PROJECT_ID` | GCP project for deploy |
| `GCP_REGION` | e.g. `us-central1` |
| `ARTIFACT_REPO` | Artifact Registry repo, default `terrarium` |
| `CLOUD_RUN_SERVICE` | Service name, default `terrarium-server` |

Frontend local config is written to `apps/skin/.env.local` by `setup-dev.sh` (gitignored).

## GitHub Actions secrets

| Secret | Purpose |
|--------|---------|
| `GCP_PROJECT_ID` | GCP project id |
| `GCP_REGION` | Deploy region |
| `CLOUD_RUN_SERVICE` | Cloud Run service name |
| `ARTIFACT_REPO` | Docker repo name |
| `FIREBASE_PROJECT_ID` | Firebase project id |
| `FIREBASE_API_KEY` | Firebase web SDK (frontend build) |
| `FIREBASE_AUTH_DOMAIN` | Firebase web SDK |
| `FIREBASE_APP_ID` | Firebase web SDK |
| `GCP_WORKLOAD_IDENTITY_PROVIDER` | WIF provider resource name |
| `GCP_SERVICE_ACCOUNT` | e.g. `github-deploy@PROJECT.iam.gserviceaccount.com` |
| `FIREBASE_SERVICE_ACCOUNT` | JSON key for Firebase Hosting deploy action |

Never commit `.env`, service account JSON, or Firebase keys.
