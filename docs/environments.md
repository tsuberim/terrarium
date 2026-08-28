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

## Deploy (when GCP is wired)

Workflows on `main` (staging) and on tags `v*` (prod) authenticate with **Workload Identity Federation** and then `gcloud storage cp` the skin. They use repository variables `GCP_WIF_PROVIDER` and `GCP_SERVICE_ACCOUNT`. If those are not set yet, the copy job is skipped; CI still has to be green.

GitHub Actions must use WIF. Never commit keys. Never paste a service-account JSON into GitHub Secrets.

## Secrets

Operator keys live only in `~/keys/` on the operator machine. See [secrets.md](secrets.md).
