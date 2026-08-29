#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ -f .env ]]; then
  set -a
  # shellcheck disable=SC1091
  source .env
  set +a
fi

: "${FIREBASE_PROJECT_ID:?Set FIREBASE_PROJECT_ID in .env}"

mkdir -p data
export LISTEN_ADDR="${LISTEN_ADDR:-0.0.0.0:8080}"
export DATABASE_URL="${DATABASE_URL:-sqlite:data/terrarium.db}"
export FAUCET_ENABLED="${FAUCET_ENABLED:-true}"

exec cargo run -p terrarium-server --bin terrarium-server
