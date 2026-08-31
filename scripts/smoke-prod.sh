#!/usr/bin/env bash
set -euo pipefail

# Post-deploy smoke: Cloud Run direct + Firebase Hosting /api rewrite.
# Retries health until Cloud Run cold start completes.

CLOUD_RUN_URL="${CLOUD_RUN_URL:?}"
HOSTING_URL="${HOSTING_URL:?}"
MAX_ATTEMPTS="${SMOKE_ATTEMPTS:-36}"
SLEEP_SECS="${SMOKE_SLEEP_SECS:-5}"

cloud_run_url="${CLOUD_RUN_URL%/}"
hosting_url="${HOSTING_URL%/}"

check_json() {
  local url="$1"
  curl -sf "$url" | python3 -c "
import json, sys
data = json.load(sys.stdin)
assert data.get('status') == 'ok', data
print('ok:', sys.argv[1])
" "$url"
}

wait_for() {
  local label="$1"
  local url="$2"
  local attempt=1
  while (( attempt <= MAX_ATTEMPTS )); do
    if check_json "$url" 2>/dev/null; then
      return 0
    fi
    echo "[$label] attempt $attempt/$MAX_ATTEMPTS — waiting ${SLEEP_SECS}s ($url)"
    sleep "$SLEEP_SECS"
    attempt=$((attempt + 1))
  done
  echo "[$label] failed after $MAX_ATTEMPTS attempts: $url" >&2
  return 1
}

echo "==> Cloud Run health (direct)"
wait_for "cloud-run" "${cloud_run_url}/api/health"

echo "==> Hosting health (/api rewrite)"
wait_for "hosting" "${hosting_url}/api/health"

echo "==> World HTTP"
curl -sf "${hosting_url}/api/v1/world" | python3 -c "
import json, sys
data = json.load(sys.stdin)
assert 'deploy_cost' in data and 'creatures' in data, data
print('world ok:', len(data['creatures']), 'creatures')
"

echo "Smoke passed."
