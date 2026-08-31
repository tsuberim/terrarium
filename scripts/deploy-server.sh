#!/usr/bin/env bash
set -euo pipefail

# Build, push, and deploy the API to Cloud Run (scales to zero when idle).
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

export DOCKER_BUILDKIT=1

if [[ -n "${GITHUB_ACTIONS:-}" ]]; then
  docker buildx create --use --name terrarium-builder 2>/dev/null || docker buildx use terrarium-builder
  docker buildx build --platform linux/amd64 \
    --cache-from type=gha \
    --cache-to type=gha,mode=max \
    -t "$IMAGE" \
    --push \
    .
else
  docker build --platform linux/amd64 -t "$IMAGE" .
  docker push "$IMAGE"
fi

ENV_VARS="FIREBASE_PROJECT_ID=${FIREBASE_PROJECT_ID},FAUCET_ENABLED=true,DATABASE_URL=sqlite:///app/data/terrarium.db?mode=rwc"

gcloud run deploy "$CLOUD_RUN_SERVICE" \
  --project="$GCP_PROJECT_ID" \
  --image="$IMAGE" \
  --region="$GCP_REGION" \
  --platform=managed \
  --port=8080 \
  --min-instances=0 \
  --max-instances=2 \
  --memory=2Gi \
  --allow-unauthenticated \
  --set-env-vars="$ENV_VARS" \
  --add-volume=name=terrarium-data,type=in-memory,size-limit=512Mi \
  --add-volume-mount=volume=terrarium-data,mount-path=/app/data

URL="$(gcloud run services describe "$CLOUD_RUN_SERVICE" \
  --project="$GCP_PROJECT_ID" \
  --region="$GCP_REGION" \
  --format='value(status.url)')"

mkdir -p .deploy
echo "$URL" > .deploy/cloud-run-url

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  echo "cloud_run_url=$URL" >> "$GITHUB_OUTPUT"
fi

echo "Cloud Run deployed: ${URL}/api/health"
echo "WebSocket (direct): ${URL/https:/wss:}/api/v1/world/ws"
