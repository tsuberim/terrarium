#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

PROJECT="${FIREBASE_PROJECT_ID:-terrarium-506917}"
APP_ID="${FIREBASE_APP_ID:-1:207363354287:web:0843f6cf3495a05818ee5a}"

if [[ ! -f .env ]]; then
  cp .env.example .env
  # shellcheck disable=SC2016
  sed -i '' "s/your-firebase-project-id/${PROJECT}/" .env 2>/dev/null \
    || sed -i "s/your-firebase-project-id/${PROJECT}/" .env
  echo "Created .env"
fi

echo "Fetching Firebase web config…"
firebase apps:sdkconfig WEB "$APP_ID" --project "$PROJECT" 2>/dev/null | python3 -c "
import json, sys
cfg = json.load(sys.stdin)
print('VITE_API_BASE=')
print(f\"VITE_FIREBASE_API_KEY={cfg['apiKey']}\")
print(f\"VITE_FIREBASE_AUTH_DOMAIN={cfg['authDomain']}\")
print(f\"VITE_FIREBASE_PROJECT_ID={cfg['projectId']}\")
print(f\"VITE_FIREBASE_APP_ID={cfg['appId']}\")
" > apps/skin/.env.local

echo "Wrote apps/skin/.env.local"
echo "Installing frontend deps…"
npm --prefix apps/skin install

if ! command -v cargo-watch >/dev/null 2>&1; then
  echo "Installing cargo-watch…"
  cargo install cargo-watch
fi

echo ""
echo "Dev ready:"
echo "  ./scripts/dev-bg.sh   # watch mode, background"
echo "  ./scripts/dev.sh      # watch mode, foreground"
