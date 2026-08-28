# Environments

GCP project: **`terrarium-506917`**. Region: **`us-central1`**.

Staging and prod are **Cloud Run** services that run the native host (persistent `World`). A static bucket is not a process and is not the deploy target.

| Name | Cloud Run service | Notes |
| --- | --- | --- |
| Staging | `terrarium-staging` | `TERRARIUM_ENV=staging`, deploy on push to `main` |
| Production | `terrarium-prod` | `TERRARIUM_ENV=prod`, deploy on tags `v*` |

Public HTTPS URLs come from Cloud Run after the first deploy (`gcloud run services describe … --format='value(status.url)'`). There is no separate HTTPS load balancer.

### Service shape (cost rule: cheapest that works)

- **min instances: 1** — scale-to-zero would wipe in-memory World state
- **max instances: 1** — one authoritative World (no split-brain)
- **memory: 128Mi**, **cpu: 1**
- **`--no-cpu-throttling`** — tick loop must run between HTTP/WS requests
- Allow unauthenticated (public skin + WS)

No GKE, GCE VMs, Cloud SQL, or extra load balancers for this milestone.

## Local

```bash
cargo run -p terrarium-host
# open http://127.0.0.1:8080/
```

Optional WASM rebuild (not required for live play):

```bash
./scripts/build-wasm.sh
```

## CI

GitHub Actions, on pull requests and on push to `main`:

1. `cargo test -p terrarium-kernel`
2. `cargo build -p terrarium-host`
3. Required docs files exist

Deploys are not part of the test workflow.

## Deploy (when GCP is wired)

Workflows authenticate with **Workload Identity Federation** (`GCP_WIF_PROVIDER`, `GCP_SERVICE_ACCOUNT`) and run `gcloud run deploy` with `--source=.` (Dockerfile at repo root). If those variables are unset, the deploy job is skipped; CI still has to be green.

The WIF service account needs permission to build and deploy Cloud Run (Cloud Build / Artifact Registry / Run admin as usual for `--source` deploys).

GitHub Actions must use WIF. Never commit keys. Never paste a service-account JSON into GitHub Secrets.

## Secrets

Operator keys live only in `~/keys/` on the operator machine. See [secrets.md](secrets.md).
