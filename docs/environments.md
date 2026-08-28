# Environments

GCP project: **`terrarium-506917`**. Region: **`us-central1`**.

This milestone is cheap on purpose. The **skin** is static files on public GCS buckets. The **API + dashboard** run as one lightweight process (local dev or a single Cloud Run service on staging — not a farm).

| Name | Skin (GCS) | API (when deployed) |
| --- | --- | --- |
| Staging | https://storage.googleapis.com/terrarium-506917-staging/index.html | Cloud Run `terrarium-api-staging` (optional; see below) |
| Production | https://storage.googleapis.com/terrarium-506917-prod/index.html | Cloud Run `terrarium-api-prod` (optional; faucet **off**) |
| Local | http://127.0.0.1:8080/ (`python3 -m http.server --directory apps/skin`) | http://127.0.0.1:3000/ (`./scripts/run-api.sh`; faucet **on**) |

| Bucket | Purpose |
| --- | --- |
| `gs://terrarium-506917-staging` | Skin static files |
| `gs://terrarium-506917-prod` | Skin static files |

The skin lives in `apps/skin/` (`index.html`, `styles.css`, `main.js`, `pkg/` WASM). Deploy is a copy:

```bash
gcloud storage cp -r apps/skin/* gs://terrarium-506917-staging --cache-control="public, max-age=60"
gcloud storage cp -r apps/skin/* gs://terrarium-506917-prod --cache-control="public, max-age=60"
```

Rebuild the kernel WASM before deploy when the crate changes:

```bash
./scripts/build-wasm.sh
```
Objects must be publicly readable. Website configuration can wait; the `index.html` URLs above are enough.

## API (local)

```bash
cp .env.example .env   # TERRARIUM_ENV=local → free mint on
./scripts/run-api.sh   # http://127.0.0.1:3000/dashboard/
```

QA flow: open dashboard → create account → faucet credits → mint API token → `curl -X POST …/v1/spawn` with Bearer token.

## API (staging deploy — optional)

One Cloud Run service, scale-to-zero, SQLite on `/tmp` (ephemeral — fine for staging QA). Set `TERRARIUM_ENV=staging` so the faucet works. No Stripe keys required.

```bash
# from repo root, after cargo build
gcloud run deploy terrarium-api-staging \
  --source crates/api \
  --region us-central1 \
  --project terrarium-506917 \
  --set-env-vars TERRARIUM_ENV=staging \
  --allow-unauthenticated
```

Production deploy is the same with `TERRARIUM_ENV=production` (faucet disabled).

## CI

GitHub Actions, on pull requests and on push to `main`:

1. `cargo test --manifest-path crates/kernel/Cargo.toml`
2. `cargo test --manifest-path crates/api/Cargo.toml`
3. Required docs files exist

That is all CI does. No image builds. No deploys inside the test workflow.

Failures post to **`#ci`** (`C0BT9UDTQJX`) when the `SLACK_BOT_TOKEN` repository secret is set (Remy bot token; see below).

## Deploy (when GCP is wired)

Workflows on `main` (staging) and on tags `v*` (prod) authenticate with **Workload Identity Federation** and then `gcloud storage cp` the skin. They use repository variables `GCP_WIF_PROVIDER` and `GCP_SERVICE_ACCOUNT`, and pass `audience: https://github.com/tsuberim/terrarium` to `google-github-actions/auth@v2` (must match the GCP OIDC provider's allowed audiences). If the variables are not set yet, the copy job is skipped; CI still has to be green.

GitHub Actions must use WIF. Never commit keys. Never paste a service-account JSON into GitHub Secrets.

Deploy results (success or failure) post to **`#deploys`** (`C0BT9UDEVS7`) when `SLACK_BOT_TOKEN` is set.

## Slack (CI and deploys)

GitHub Actions post as **Remy** via the Slack Web API (`chat.postMessage`). No incoming webhooks.

| Secret | Bot | Channels |
| --- | --- | --- |
| `SLACK_BOT_TOKEN` | Remy | `#ci`, `#deploys` |

Add the secret under GitHub → repo → Settings → Secrets and variables → Actions. The operator bot token lives in `~/keys/slack-remy.bot` (see [secrets.md](secrets.md)) — never commit it. The composite action at `.github/actions/slack-notify` skips quietly if the secret is missing.

## Secrets

Operator keys live only in `~/keys/` on the operator machine. See [secrets.md](secrets.md).
