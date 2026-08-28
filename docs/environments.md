# Environments

GCP project: **`terrarium-506917`**. Region: **`us-central1`**.

This milestone is cheap on purpose. Staging and prod are two public Cloud Storage buckets serving static files over HTTPS. There is no Cloud Run, no Docker, no Artifact Registry, no GKE, no VMs, and no load balancer.

| Name | Bucket | Public URL |
| --- | --- | --- |
| Staging | `gs://terrarium-506917-staging` | https://storage.googleapis.com/terrarium-506917-staging/index.html |
| Production | `gs://terrarium-506917-prod` | https://storage.googleapis.com/terrarium-506917-prod/index.html |

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

## CI

GitHub Actions, on pull requests and on push to `main`:

1. `cargo test --manifest-path crates/kernel/Cargo.toml`
2. Required docs files exist

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
