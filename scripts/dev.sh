#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

[[ -f .env ]] && set -a && source .env && set +a
: "${FIREBASE_PROJECT_ID:?Run ./scripts/setup-dev.sh first}"
[[ -f apps/skin/.env.local ]] || { echo "Run ./scripts/setup-dev.sh first"; exit 1; }
command -v cargo-watch >/dev/null || { echo "Run ./scripts/setup-dev.sh first"; exit 1; }

export LISTEN_ADDR="${LISTEN_ADDR:-0.0.0.0:8080}"
export DATABASE_URL="${DATABASE_URL:-sqlite:data/terrarium.db}"
export FAUCET_ENABLED="${FAUCET_ENABLED:-true}"
export DEV_MODE="${DEV_MODE:-true}"

mkdir -p data

echo "Local API: http://127.0.0.1:8080/api/health"
echo "UI:        http://localhost:5173  (proxies /api → local server)"
echo "API docs:  http://localhost:5173/api/docs"
echo ""

exec npm run dev
