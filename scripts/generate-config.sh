#!/usr/bin/env bash
set -euo pipefail

# Writes apps/skin/.env.production for Vite build (CI + manual deploy).

cd "$(dirname "$0")/.."

: "${FIREBASE_API_KEY:?}"
: "${FIREBASE_AUTH_DOMAIN:?}"
: "${FIREBASE_PROJECT_ID:?}"
: "${FIREBASE_APP_ID:?}"

API_BASE="${TERRARIUM_API_BASE:-}"

cat > apps/skin/.env.production <<EOF
VITE_API_BASE=${API_BASE}
VITE_FIREBASE_API_KEY=${FIREBASE_API_KEY}
VITE_FIREBASE_AUTH_DOMAIN=${FIREBASE_AUTH_DOMAIN}
VITE_FIREBASE_PROJECT_ID=${FIREBASE_PROJECT_ID}
VITE_FIREBASE_APP_ID=${FIREBASE_APP_ID}
EOF

echo "Wrote apps/skin/.env.production"
