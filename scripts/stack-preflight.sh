#!/usr/bin/env bash
# Verify dev stack is up before smoke or e2e.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=health-check.sh
source "$ROOT/scripts/health-check.sh"

API="${API_BASE:-http://127.0.0.1:8080/api}"
COMPILE="${COMPILE_WORKER_URL:-http://127.0.0.1:8081}"
AUTH_HOST="${FIREBASE_AUTH_EMULATOR_HOST:-127.0.0.1:9099}"
UI="${E2E_UI_URL:-http://localhost:5173}"
REQUIRE_UI="${STACK_PREFLIGHT_UI:-true}"

fail() {
  echo "stack-preflight: $1" >&2
  exit 1
}

echo "==> API ${API}/health"
health_check_api "$API" || fail "API down at ${API}"

echo "==> Compile worker ${COMPILE}/health"
health_check_compile "$COMPILE" || fail "compile worker down or missing body_wrap"

echo "==> Auth emulator ${AUTH_HOST}"
health_check_auth_emulator "$AUTH_HOST" || fail "auth emulator identity API down at ${AUTH_HOST}"

if [[ "$REQUIRE_UI" == "true" ]]; then
  echo "==> UI ${UI}"
  curl -sf "${UI}" >/dev/null || fail "Vite UI down at ${UI}"
  echo ""
  echo "Ready: ${UI}"
  echo "E2E bridge: window.__TERRARIUM_E2E__.getState()"
else
  echo ""
  echo "API stack ready (UI not required)"
fi
