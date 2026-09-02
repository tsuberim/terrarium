#!/usr/bin/env bash
# Build and deploy the isolated Rust compile worker to Cloud Run (min-instances=0).
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ -f .env ]]; then
  set -a
  # shellcheck disable=SC1091
  source .env
  set +a
fi

: "${GCP_PROJECT_ID:?}"
: "${GCP_REGION:=us-central1}"
: "${ARTIFACT_REPO:=terrarium}"
: "${COMPILE_RUN_SERVICE:=terrarium-compile}"

IMAGE="${GCP_REGION}-docker.pkg.dev/${GCP_PROJECT_ID}/${ARTIFACT_REPO}/compile-worker:${GITHUB_SHA:-latest}"

gcloud auth configure-docker "${GCP_REGION}-docker.pkg.dev" --quiet

export DOCKER_BUILDKIT=1

if [[ -n "${GITHUB_ACTIONS:-}" ]]; then
  docker buildx create --use --name terrarium-compile-builder 2>/dev/null || docker buildx use terrarium-compile-builder
  docker buildx build --platform linux/amd64 \
    --cache-from type=gha,scope=compile-worker \
    --cache-to type=gha,mode=max,scope=compile-worker \
    -f services/compile-worker/Dockerfile \
    -t "$IMAGE" \
    --push \
    .
else
  docker build --platform linux/amd64 -f services/compile-worker/Dockerfile -t "$IMAGE" .
  docker push "$IMAGE"
fi

gcloud run deploy "$COMPILE_RUN_SERVICE" \
  --project="$GCP_PROJECT_ID" \
  --image="$IMAGE" \
  --region="$GCP_REGION" \
  --platform=managed \
  --port=8080 \
  --min-instances=0 \
  --max-instances=3 \
  --memory=2Gi \
  --cpu=2 \
  --timeout=120 \
  --concurrency=1 \
  --no-allow-unauthenticated

URL="$(gcloud run services describe "$COMPILE_RUN_SERVICE" \
  --project="$GCP_PROJECT_ID" \
  --region="$GCP_REGION" \
  --format='value(status.url)')"

# Allow the API Cloud Run runtime SA to invoke this worker.
if [[ -z "${COMPILE_INVOKER_SA:-}" ]]; then
  PROJECT_NUMBER="$(gcloud projects describe "$GCP_PROJECT_ID" --format='value(projectNumber)')"
  COMPILE_INVOKER_SA="${PROJECT_NUMBER}-compute@developer.gserviceaccount.com"
fi
gcloud run services add-iam-policy-binding "$COMPILE_RUN_SERVICE" \
  --project="$GCP_PROJECT_ID" \
  --region="$GCP_REGION" \
  --member="serviceAccount:${COMPILE_INVOKER_SA}" \
  --role="roles/run.invoker" \
  --quiet
echo "Granted run.invoker on ${COMPILE_RUN_SERVICE} to ${COMPILE_INVOKER_SA}"

mkdir -p .deploy
echo "$URL" > .deploy/compile-worker-url

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  echo "compile_worker_url=$URL" >> "$GITHUB_OUTPUT"
fi

echo "Compile worker deployed (internal): $URL"
echo "Set COMPILE_WORKER_URL on terrarium-server to this URL."
