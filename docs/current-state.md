# Current state

Honest snapshot as of 2026-08-28.

**There is no simulation yet.** No physics, no WASM guests, no verbs beyond mass accounting, no multiplayer, no cash rail.

This milestone is architecture, CI, and a staging/prod shell:

- Docs in `/docs` hold vision and architecture.
- `crates/kernel` is a mass ledger (`Mass`, `World`, spawn / spend / dump / absorb) with tests that conservation holds except `spend` (house burn).
- `apps/skin` is a static landing/camera. It does not talk to a kernel.
- CI runs `cargo test` and checks that required docs exist.
- Staging and prod will be two public GCS buckets serving that skin. They may be empty until the first `gcloud storage cp`.
- WIF is not wired in this commit. Deploy jobs skip until `GCP_WIF_PROVIDER` and `GCP_SERVICE_ACCOUNT` exist.

What this is not: a Cloud Run service, a container, a cluster, a fake demo of creatures eating each other.
