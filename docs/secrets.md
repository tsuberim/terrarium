# Secrets

All keys live in **`~/keys/`** on the operator machine. That path is outside the repo. It is never copied into this tree, never committed, never attached to a cloud agent workspace as a "convenience."

## Never

- Keys in the git repo (including `/keys`, `.env`, `*.pem`, service-account JSON).
- Keys in chat logs, if avoidable. Do not paste JSON key files into Cursor, Slack, or a ticket.
- GitHub Secrets as copies of the service-account JSON. That duplicates a key we already refused to store. Use **Workload Identity Federation** instead.
- Service account JSON in CI checkout, Docker build context (there is no Docker in this milestone), or Cloud Storage next to the skin.

## How CI talks to GCP

GitHub Actions authenticates as a service account through WIF:

- Variable `GCP_WIF_PROVIDER` — the provider resource name
- Variable `GCP_SERVICE_ACCOUNT` — the service account email

Those are identifiers, not key files. The JSON private key never leaves Google, never sits in GitHub Secrets, never sits in this repo.

## Operator machine

If you need to run `gcloud` yourself, use whatever is already in `~/keys/` on that machine. Do not invent a second copy "for the project." The repo's `.gitignore` refuses `/keys`, `.env`, `*.pem`, and service-account JSON patterns as a backstop, not as a place to put files.
