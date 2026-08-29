#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

[[ -f .env ]] && set -a && source .env && set +a
: "${FIREBASE_PROJECT_ID:?Run ./scripts/setup-dev.sh first}"
[[ -f apps/skin/.env.local ]] || { echo "Run ./scripts/setup-dev.sh first"; exit 1; }
command -v cargo-watch >/dev/null || { echo "Run ./scripts/setup-dev.sh first"; exit 1; }

export LISTEN_ADDR="${LISTEN_ADDR:-0.0.0.0:8080}"
export DATABASE_URL="${DATABASE_URL:-sqlite::memory:}"
export FAUCET_ENABLED="${FAUCET_ENABLED:-true}"

exec npm run dev
