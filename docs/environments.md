# Environments

Single **prod** environment on GCP + Firebase. No staging for now.

See [devops.md](devops.md) for the full picture (local dev, CI/CD, scripts).

## Production stack

| Component | Platform | URL |
|-----------|----------|-----|
| World server | Cloud Run (`terrarium-server`) | via Hosting rewrite `/api/**` |
| Spectator UI | Firebase Hosting | https://terrarium-506917.web.app |
| Auth | Firebase Auth | same project |
| Database | SQLite in-memory on Cloud Run | ephemeral — resets on redeploy |

## One-time GCP setup

Replace `PROJECT` and `REGION` (default `us-central1`).

```bash
gcloud config set project PROJECT

# APIs
gcloud services enable \
  run.googleapis.com \
  artifactregistry.googleapis.com \
  iamcredentials.googleapis.com

# Artifact Registry
gcloud artifacts repositories create terrarium \
  --repository-format=docker \
  --location=REGION

# Service account for GitHub Actions
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

# Workload Identity Federation pool + provider for your GitHub repo
# (see https://github.com/google-github-actions/auth)
```

First manual deploy to create the Cloud Run service:

```bash
cp .env.example .env   # fill GCP_* and FIREBASE_PROJECT_ID
./scripts/deploy-server.sh
```

## One-time Firebase setup

```bash
npm install -g firebase-tools
firebase login
firebase use terrarium-506917
```

In Firebase Console → Authentication:

- Enable **Google** (and/or Email) sign-in
- Add `localhost` to authorized domains for local dev

Deploy hosting (or let CI do it on push to `main`):

```bash
./scripts/generate-config.sh   # or use setup-dev.sh values
npm ci && npm run build        # in apps/skin
firebase deploy --only hosting
```

## Local

```bash
./scripts/setup-dev.sh
./scripts/dev-bg.sh    # or ./scripts/dev.sh
```

Open http://127.0.0.1:5173 — Vite proxies `/api` to `:8080`.

## Credits faucet

`FAUCET_ENABLED=true` by default. Any authenticated Firebase user can POST `/v1/faucet` up to `FAUCET_MAX` credits per request. Disable when real payments land.
