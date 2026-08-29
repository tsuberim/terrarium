#!/usr/bin/env bash
set -euo pipefail

# Build, push, and deploy the API to Cloud Run (min-instances=1).
# Firebase Hosting rewrites /api/** to this service.

cd "$(dirname "$0")/.."

if [[ -f .env ]]; then
  set -a
  # shellcheck disable=SC1091
  source .env
  set +a
fi

: "${GCP_PROJECT_ID:?}"
: "${GCP_REGION:=us-central1}"
: "${CLOUD_RUN_SERVICE:=terrarium-server}"
: "${ARTIFACT_REPO:=terrarium}"
: "${FIREBASE_PROJECT_ID:?}"

IMAGE="${GCP_REGION}-docker.pkg.dev/${GCP_PROJECT_ID}/${ARTIFACT_REPO}/server:${GITHUB_SHA:-latest}"

gcloud auth configure-docker "${GCP_REGION}-docker.pkg.dev" --quiet

docker build --platform linux/amd64 -t "$IMAGE" .
docker push "$IMAGE"

gcloud run deploy "$CLOUD_RUN_SERVICE" \
  --project="$GCP_PROJECT_ID" \
  --image="$IMAGE" \
  --region="$GCP_REGION" \
  --platform=managed \
  --port=8080 \
  --min-instances=1 \
  --max-instances=2 \
  --allow-unauthenticated \
  --set-env-vars="FIREBASE_PROJECT_ID=${FIREBASE_PROJECT_ID},FAUCET_ENABLED=true,DATABASE_URL=sqlite::memory:"

URL="$(gcloud run services describe "$CLOUD_RUN_SERVICE" \
  --project="$GCP_PROJECT_ID" \
  --region="$GCP_REGION" \
  --format='value(status.url)')"

echo "Cloud Run deployed: ${URL}/api/health"
