#!/usr/bin/env bash
set -euo pipefail

# Writes apps/skin/.env.production for Vite build (CI + manual deploy).

cd "$(dirname "$0")/.."

: "${FIREBASE_API_KEY:?}"
: "${FIREBASE_AUTH_DOMAIN:?}"
: "${FIREBASE_PROJECT_ID:?}"
: "${FIREBASE_APP_ID:?}"

API_BASE="${TERRARIUM_API_BASE:-}"

ws_base="${TERRARIUM_WS_BASE:-}"
if [[ -z "$ws_base" && -f .deploy/cloud-run-url ]]; then
  ws_base=$(cat .deploy/cloud-run-url)
fi
if [[ -z "$ws_base" ]]; then
  echo "error: TERRARIUM_WS_BASE or .deploy/cloud-run-url required (Firebase Hosting cannot proxy WebSocket)" >&2
  exit 1
fi
if [[ -n "$ws_base" ]]; then
  ws_base="${ws_base%/}/api"
  ws_base="${ws_base/https:/wss:}"
  ws_base="${ws_base/http:/ws:}"
fi

cat > apps/skin/.env.production <<EOF
VITE_API_BASE=${API_BASE}
VITE_WS_BASE=${ws_base}
VITE_FIREBASE_API_KEY=${FIREBASE_API_KEY}
VITE_FIREBASE_AUTH_DOMAIN=${FIREBASE_AUTH_DOMAIN}
VITE_FIREBASE_PROJECT_ID=${FIREBASE_PROJECT_ID}
VITE_FIREBASE_APP_ID=${FIREBASE_APP_ID}
EOF

echo "Wrote apps/skin/.env.production"
