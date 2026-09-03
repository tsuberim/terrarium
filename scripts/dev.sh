#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

[[ -f .env ]] && set -a && source .env && set +a
: "${FIREBASE_PROJECT_ID:?Run ./scripts/setup-dev.sh first}"
[[ -f apps/skin/.env.local ]] || { echo "Run ./scripts/setup-dev.sh first"; exit 1; }
command -v cargo-watch >/dev/null || { echo "Run ./scripts/setup-dev.sh first"; exit 1; }

export LISTEN_ADDR="${LISTEN_ADDR:-0.0.0.0:8080}"
export DATABASE_URL="${DATABASE_URL:-sqlite://data/terrarium.db?mode=rwc}"
export CARGO_TARGET_DIR="$PWD/target"
export FAUCET_ENABLED="${FAUCET_ENABLED:-true}"
export DEV_MODE="${DEV_MODE:-true}"
export COMPILE_WORKER_URL="${COMPILE_WORKER_URL:-http://127.0.0.1:8081}"
export FIREBASE_AUTH_EMULATOR_HOST="${FIREBASE_AUTH_EMULATOR_HOST:-127.0.0.1:9099}"

mkdir -p data

"$(dirname "$0")/dev-stop.sh" >/dev/null

echo "Local API: http://127.0.0.1:8080/api/health"
echo "Auth emu:  http://127.0.0.1:9099  (auto sign-in as qa@terrarium.dev)"
echo "Compile:   ${COMPILE_WORKER_URL}/health"
echo "UI:        http://localhost:5173  (proxies /api → local server)"
echo "Headless:  npm run qa"
echo "Browser:   npm run qa:e2e  (Playwright)"
echo "Preflight: npm run qa:preflight"
echo "QA docs:   docs/internal/qa/README.md"
echo "API docs:  http://localhost:5173/api/docs"
echo ""

exec npm run dev
