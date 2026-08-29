#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ -f .env ]]; then
  set -a
  # shellcheck disable=SC1091
  source .env
  set +a
fi

: "${FIREBASE_PROJECT_ID:?Run ./scripts/setup-dev.sh first}"

if [[ ! -f apps/skin/.env.local ]]; then
  echo "Missing apps/skin/.env.local — run ./scripts/setup-dev.sh"
  exit 1
fi

if ! command -v cargo-watch >/dev/null 2>&1; then
  echo "cargo-watch required — run ./scripts/setup-dev.sh"
  exit 1
fi

export LISTEN_ADDR="${LISTEN_ADDR:-0.0.0.0:8080}"
export DATABASE_URL="${DATABASE_URL:-sqlite::memory:}"
export FAUCET_ENABLED="${FAUCET_ENABLED:-true}"

SERVER_PID=""

cleanup() {
  if [[ -n "$SERVER_PID" ]]; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

echo "API watch → http://localhost:8080"
cargo watch -q -x 'run -p terrarium-server --bin terrarium-server' &
SERVER_PID=$!

for _ in $(seq 1 15); do
  curl -sf "http://127.0.0.1:8080/health" >/dev/null 2>&1 && break
  sleep 1
done

echo "Frontend watch → http://localhost:5173"
cd apps/skin
exec npm run dev
