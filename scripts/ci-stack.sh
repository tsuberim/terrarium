#!/usr/bin/env bash
# Shared CI stack bootstrap — sourced by ci-api-smoke.sh and ci-e2e.sh.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
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

CI_STACK_PIDS=()

ci_stack_cleanup() {
  for pid in "${CI_STACK_PIDS[@]}"; do
    kill "$pid" 2>/dev/null || true
  done
  wait 2>/dev/null || true
}

ci_stack_write_env() {
  local e2e_hooks="${1:-false}"
  cat > apps/skin/.env.local <<EOF
VITE_API_BASE=
VITE_WS_BASE=
VITE_FIREBASE_API_KEY=fake-api-key
VITE_FIREBASE_AUTH_DOMAIN=${FIREBASE_PROJECT_ID}.firebaseapp.com
VITE_FIREBASE_PROJECT_ID=${FIREBASE_PROJECT_ID}
VITE_FIREBASE_APP_ID=ci-app
VITE_USE_AUTH_EMULATOR=true
VITE_E2E_HOOKS=${e2e_hooks}
EOF
}

ci_stack_build() {
  echo "==> Build Rust (server + compile-worker)"
  rustup target add wasm32-unknown-unknown
  cargo build --release -q -p terrarium-server
  cargo build --release -q --manifest-path services/compile-worker/Cargo.toml
}

ci_stack_start_api() {
  mkdir -p data
  trap ci_stack_cleanup EXIT

  echo "==> Start auth emulator"
  npx --yes firebase-tools emulators:start --only auth --project "$FIREBASE_PROJECT_ID" &
  CI_STACK_PIDS+=($!)

  echo "==> Start compile-worker"
  (
    export TEMPLATE_DIR="$ROOT/services/compile-worker/template"
    export SDK_PATH="$ROOT/sdk/rust/terrarium-sdk"
    export PORT="$COMPILE_PORT"
    exec "$CARGO_TARGET_DIR/release/compile-worker"
  ) &
  CI_STACK_PIDS+=($!)

  echo "==> Start API server"
  "$CARGO_TARGET_DIR/release/terrarium-server" &
  CI_STACK_PIDS+=($!)
}

ci_stack_start_ui() {
  echo "==> Install frontend"
  npm ci --prefix apps/skin

  echo "==> Start Vite"
  npm run dev --prefix apps/skin &
  CI_STACK_PIDS+=($!)
}

ci_stack_install_playwright() {
  echo "==> Install Playwright"
  npx --prefix apps/skin playwright install --with-deps chromium
}

ci_stack_wait() {
  local require_ui="${1:-false}"
  echo "==> Wait for stack"
  export STACK_PREFLIGHT_UI="$require_ui"
  for _ in $(seq 1 90); do
    if ./scripts/stack-preflight.sh >/dev/null 2>&1; then
      break
    fi
    sleep 2
  done
  ./scripts/stack-preflight.sh
}
