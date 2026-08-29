#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

PROJECT="${FIREBASE_PROJECT_ID:-terrarium-506917}"
APP_ID="${FIREBASE_APP_ID:-1:207363354287:web:0843f6cf3495a05818ee5a}"

[[ -f .env ]] || cp .env.example .env

echo "Firebase web config…"
firebase apps:sdkconfig WEB "$APP_ID" --project "$PROJECT" 2>/dev/null | python3 -c "
import json, sys
cfg = json.load(sys.stdin)
print('# Local dev — API + WS via Vite proxy → localhost:8080')
print('VITE_API_BASE=')
print('VITE_WS_BASE=')
print(f\"VITE_FIREBASE_API_KEY={cfg['apiKey']}\")
print(f\"VITE_FIREBASE_AUTH_DOMAIN={cfg['authDomain']}\")
print(f\"VITE_FIREBASE_PROJECT_ID={cfg['projectId']}\")
print(f\"VITE_FIREBASE_APP_ID={cfg['appId']}\")
" > apps/skin/.env.local

mkdir -p data
grep -q '^DEV_MODE=' .env 2>/dev/null || echo 'DEV_MODE=true' >> .env
grep -q '^DATABASE_URL=' .env 2>/dev/null || echo 'DATABASE_URL=sqlite:data/terrarium.db' >> .env

npm install
npm --prefix apps/skin install
command -v cargo-watch >/dev/null || cargo install cargo-watch

echo ""
echo "Run: ./scripts/dev.sh"
echo "Open: http://localhost:5173"
