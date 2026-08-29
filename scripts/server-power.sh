#!/usr/bin/env bash
# Set Cloud Run min-instances (wake / sleep the backend).
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
: "${CLOUD_RUN_SERVICE:=terrarium-server}"

MIN_ON="${SERVER_MIN_INSTANCES_ON:-1}"

usage() {
  echo "Usage: $0 on|off|status"
  echo "  on     min-instances=${MIN_ON} (sim stays warm)"
  echo "  off    min-instances=0 (scales to zero when idle — saves cost)"
  echo "  status show current min-instances"
  exit 1
}

[[ $# -eq 1 ]] || usage

case "$1" in
  on)
    MIN="$MIN_ON"
    ;;
  off)
    MIN=0
    ;;
  status)
    gcloud run services describe "$CLOUD_RUN_SERVICE" \
      --project="$GCP_PROJECT_ID" \
      --region="$GCP_REGION" \
      --format='yaml(spec.template.metadata.annotations.autoscaling.knative.dev/minScale,status.url)'
    exit 0
    ;;
  *)
    usage
    ;;
esac

gcloud run services update "$CLOUD_RUN_SERVICE" \
  --project="$GCP_PROJECT_ID" \
  --region="$GCP_REGION" \
  --min-instances="$MIN"

echo "Cloud Run min-instances=${MIN}"
