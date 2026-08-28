# Environments

GCP / Firebase project: **`terrarium-506917`**. Region: **`us-central1`**.

## Destination vs legacy

| | Destination | Legacy (main today) |
| --- | --- | --- |
| Skin + dashboard static files | **Firebase Hosting** | GCS buckets (`gcloud storage cp`) |
| Sim process | **Host** (`crates/host`, PR #4) WebSocket | WASM ticks in browser tab |
| Human auth | **Firebase Auth** | Dev session tokens (`trm_sess_…`) |
| API | Cloud Run or co-located with host | `./scripts/run-api.sh` locally |

`firebase.json` + `.firebaserc` are checked in. **TODO:** switch GitHub deploy workflows from GCS to `firebase deploy` — not done on this branch.

### Firebase Hosting (config only)

```bash
# operator machine — after firebase login
firebase deploy --only hosting:skin --project terrarium-506917      # staging target TBD
firebase deploy --only hosting:dashboard --project terrarium-506917
```

Hosting targets in `firebase.json`: `skin` → `apps/skin`, `dashboard` → `apps/dashboard`.

### Legacy GCS (still wired in CI)

| Bucket | URL |
| --- | --- |
| `gs://terrarium-506917-staging` | https://storage.googleapis.com/terrarium-506917-staging/index.html |
| `gs://terrarium-506917-prod` | https://storage.googleapis.com/terrarium-506917-prod/index.html |

```bash
gcloud storage cp -r apps/skin/* gs://terrarium-506917-staging --cache-control="public, max-age=60"
./scripts/build-wasm.sh   # when kernel changed
```

## Local development

```bash
cp .env.example .env
./scripts/run-api.sh          # API + dashboard fallback at :3000
python3 -m http.server 8080 --directory apps/skin   # legacy WASM skin
```

| Service | URL | Notes |
| --- | --- | --- |
| API | http://127.0.0.1:3000/ | JSON + dev dashboard static |
| Dashboard | http://127.0.0.1:3000/dashboard/ | Dev sign-in when Firebase unset |
| Skin (legacy) | http://127.0.0.1:8080/ | WASM in-tab |

QA: dashboard → dev sign-in (or Firebase) → faucet → mint scoped token → `curl POST /v1/spawn`.

## API env vars

See `.env.example`. Key vars:

| Variable | Purpose |
| --- | --- |
| `TERRARIUM_ENV` | `local` / `staging` / `production` — controls free-credit faucet |
| `FIREBASE_PROJECT_ID` | When set, dashboard requires Firebase ID tokens |
| `FIREBASE_API_KEY`, `FIREBASE_AUTH_DOMAIN` | Public web config served to dashboard (not secrets) |
| `TERRARIUM_DEV_AUTH` | `1` (default) allows dev session tokens when Firebase unset |

Staging Cloud Run example:

```bash
gcloud run deploy terrarium-api-staging \
  --source crates/api \
  --region us-central1 \
  --project terrarium-506917 \
  --set-env-vars TERRARIUM_ENV=staging,FIREBASE_PROJECT_ID=terrarium-506917 \
  --allow-unauthenticated
```

Production: same with `TERRARIUM_ENV=production` (faucet **off**).

## CI

GitHub Actions on PR and push to `main`:

1. `cargo test -p terrarium-kernel`
2. `cargo test -p terrarium-api` (dev auth — no Firebase keys in CI)
3. Required docs exist

Failures post to **`#ci`** when `SLACK_BOT_TOKEN` is set.

## Deploy workflows (legacy GCS)

Workflows on `main` / tags `v*` use WIF + `gcloud storage cp` for skin only. See [secrets.md](secrets.md). Firebase Hosting deploy is a follow-up.

## Secrets

Operator keys in `~/keys/` only. See [secrets.md](secrets.md).
