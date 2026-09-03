#!/usr/bin/env bash
# CI: build stack, run API smoke + Playwright e2e. Used by reusable-test.yml `e2e` job.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export FIREBASE_PROJECT_ID="${FIREBASE_PROJECT_ID:-ci-project}"
export LISTEN_ADDR="${LISTEN_ADDR:-0.0.0.0:8080}"
export DATABASE_URL="${DATABASE_URL:-sqlite://data/terrarium-ci.db?mode=rwc}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
export FAUCET_ENABLED="${FAUCET_ENABLED:-true}"
export DEV_MODE="${DEV_MODE:-true}"
export COMPILE_WORKER_URL="${COMPILE_WORKER_URL:-http://127.0.0.1:8081}"
export FIREBASE_AUTH_EMULATOR_HOST="${FIREBASE_AUTH_EMULATOR_HOST:-127.0.0.1:9099}"
export COMPILE_PORT="${COMPILE_PORT:-8081}"

PIDS=()
cleanup() {
  for pid in "${PIDS[@]}"; do
    kill "$pid" 2>/dev/null || true
  done
  wait 2>/dev/null || true
}
trap cleanup EXIT

mkdir -p data

cat > apps/skin/.env.local <<EOF
VITE_API_BASE=
VITE_WS_BASE=
VITE_FIREBASE_API_KEY=fake-api-key
VITE_FIREBASE_AUTH_DOMAIN=${FIREBASE_PROJECT_ID}.firebaseapp.com
VITE_FIREBASE_PROJECT_ID=${FIREBASE_PROJECT_ID}
VITE_FIREBASE_APP_ID=ci-app
VITE_USE_AUTH_EMULATOR=true
VITE_QA_MODE=true
EOF

echo "==> Build Rust (server + compile-worker)"
rustup target add wasm32-unknown-unknown
cargo build --release -q -p terrarium-server
cargo build --release -q --manifest-path services/compile-worker/Cargo.toml

echo "==> Install frontend + Playwright"
npm ci --prefix apps/skin
npx --prefix apps/skin playwright install --with-deps chromium

echo "==> Start auth emulator"
npx --yes firebase-tools emulators:start --only auth --project "$FIREBASE_PROJECT_ID" &
PIDS+=($!)

echo "==> Start compile-worker"
(
  export TEMPLATE_DIR="$ROOT/services/compile-worker/template"
  export SDK_PATH="$ROOT/sdk/rust/terrarium-sdk"
  export PORT="$COMPILE_PORT"
  exec "$CARGO_TARGET_DIR/release/compile-worker"
) &
PIDS+=($!)

echo "==> Start API server"
"$CARGO_TARGET_DIR/release/terrarium-server" &
PIDS+=($!)

echo "==> Start Vite"
npm run dev --prefix apps/skin &
PIDS+=($!)

echo "==> Wait for stack"
for _ in $(seq 1 90); do
  if ./scripts/qa-preflight.sh >/dev/null 2>&1; then
    break
  fi
  sleep 2
done
./scripts/qa-preflight.sh || exit 1

echo "==> API smoke"
./scripts/qa-smoke.sh

echo "==> Playwright e2e"
npm run qa:e2e --prefix apps/skin

echo "CI QA passed."
