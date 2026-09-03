# Environments

**Scope:** one-time GCP/Firebase bootstrap. Not daily dev or deploy runbook.

Local dev: [../workflow/setup.md](../workflow/setup.md). Deploy: [deploy.md](deploy.md).

Single **prod** env — no staging ([tech-debt](../engineering/tech-debt.md) TD-INF-3).

---

## Production stack

| Component | Platform |
|-----------|----------|
| World server | Cloud Run — `/api/**` via Hosting rewrite |
| UI | Firebase Hosting — https://terrarium-506917.web.app |
| Auth | Firebase Auth |
| Database | Ephemeral SQLite on Cloud Run |

---

## One-time GCP setup

Replace `PROJECT` and `REGION` (default `us-central1`).

```bash
gcloud config set project PROJECT

gcloud services enable \
  run.googleapis.com \
  artifactregistry.googleapis.com \
  iamcredentials.googleapis.com

gcloud artifacts repositories create terrarium \
  --repository-format=docker \
  --location=REGION

gcloud iam service-accounts create github-deploy \
  --display-name="GitHub Deploy"

gcloud projects add-iam-policy-binding PROJECT \
  --member="serviceAccount:github-deploy@PROJECT.iam.gserviceaccount.com" \
  --role="roles/artifactregistry.writer"

gcloud projects add-iam-policy-binding PROJECT \
  --member="serviceAccount:github-deploy@PROJECT.iam.gserviceaccount.com" \
  --role="roles/run.admin"

gcloud projects add-iam-policy-binding PROJECT \
  --member="serviceAccount:github-deploy@PROJECT.iam.gserviceaccount.com" \
  --role="roles/iam.serviceAccountUser"
```

Workload Identity Federation: [google-github-actions/auth](https://github.com/google-github-actions/auth).

First deploy: `cp .env.example .env` → `./scripts/deploy-server.sh`

---

## One-time Firebase setup

```bash
npm install -g firebase-tools
firebase login
firebase use terrarium-506917
```

Console → Authentication: enable Google; add `localhost` to authorized domains.

---

## Faucet

`FAUCET_ENABLED=true` by default in dev. Disable when real payments land.
