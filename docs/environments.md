# Environments

GCP / Firebase project: **`terrarium-506917`**. Region: **`us-central1`**.

## Production shape

| Piece | Where | Notes |
| --- | --- | --- |
| Skin | Firebase Hosting (`terrarium-skin-staging` / `terrarium-skin-prod`) | WebSocket viewer to Cloud Run host |
| Dashboard | Firebase Hosting (`terrarium-dashboard-staging` / `terrarium-dashboard-prod`) | Calls Cloud Run API |
| Sim | Cloud Run **`terrarium-host-*`** | Native kernel, `min-instances=1`, `max-instances=1` |
| Credits / tokens / spawn | Cloud Run **`terrarium-api-*`** | SQLite; `TERRARIUM_HOST_URL` → host internal API |

Legacy GCS buckets (`gs://terrarium-506917-staging`, `gs://terrarium-506917-prod`) are **retired** — workflows no longer copy to them.

## Deploy (GitHub Actions)

On push to `main` (staging) and tags `v*` (prod), when WIF vars are set:

1. Deploy host to Cloud Run (always-on, 1 instance)
2. Deploy API with `TERRARIUM_HOST_URL` + `TERRARIUM_HOST_TOKEN`
3. Patch skin/dashboard HTML meta tags (`scripts/patch-hosting-meta.sh`)
4. `firebase deploy` for Hosting

Requires repository secret **`TERRARIUM_HOST_TOKEN`** (shared bearer for host internal routes). WIF via `GCP_WIF_PROVIDER` and `GCP_SERVICE_ACCOUNT` — never commit keys.

Operator manual deploy:

```bash
firebase deploy --only hosting:terrarium-skin-staging,hosting:terrarium-dashboard-staging --project terrarium-506917
gcloud run deploy terrarium-host-staging --source=. --region us-central1 --min-instances=1 --max-instances=1 ...
```

## Local development

```bash
cp .env.example .env
./scripts/run-host.sh
./scripts/run-api.sh
```

| Service | URL | Notes |
| --- | --- | --- |
| Host | http://127.0.0.1:8080/ | World + `/ws`; serves skin when `SKIN_DIR=apps/skin` |
| API | http://127.0.0.1:3000/ | JSON + dashboard static fallback |
| Skin (via host) | http://127.0.0.1:8080/ | WebSocket camera |
| Dashboard | http://127.0.0.1:3000/dashboard/ | Dev sign-in when Firebase unset |

## Env vars

See `.env.example`. Key vars:

| Variable | Purpose |
| --- | --- |
| `TERRARIUM_ENV` | `local` / `staging` / `production` — controls free-credit faucet |
| `TERRARIUM_HOST_URL` | API → host delegation (staging/prod Cloud Run URL) |
| `TERRARIUM_HOST_TOKEN` | Bearer auth on host `/internal/*` |
| `FIREBASE_PROJECT_ID` | When set, dashboard requires Firebase ID tokens |
| `TERRARIUM_DEV_AUTH` | `1` (default) allows dev session tokens when Firebase unset |

## CI

GitHub Actions on PR and push to `main`:

1. `cargo test -p terrarium-kernel`
2. `cargo test -p terrarium-api`
3. `cargo test -p terrarium-host`
4. Required docs exist

Failures post to **`#ci`** when `SLACK_BOT_TOKEN` is set.

## Secrets

Operator keys in `~/keys/` only. See [secrets.md](secrets.md).
