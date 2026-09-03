#!/usr/bin/env bash
# Verify local dev stack is up before browser or e2e QA.
set -euo pipefail

API="${API_BASE:-http://127.0.0.1:8080/api}"
COMPILE="${COMPILE_WORKER_URL:-http://127.0.0.1:8081}"
AUTH_HOST="${FIREBASE_AUTH_EMULATOR_HOST:-127.0.0.1:9099}"
UI="${QA_UI_URL:-http://localhost:5173}"

fail() {
  echo "qa-preflight: $1" >&2
  exit 1
}

echo "==> API ${API}/health"
curl -sf "${API}/health" | grep -q '"status"' || fail "API down at ${API}"

echo "==> Compile worker ${COMPILE}/health"
curl -sf "${COMPILE}/health" | python3 -c "
import json, sys
d = json.load(sys.stdin)
assert d.get('status') == 'ok' and d.get('body_wrap') is True, d
" || fail "compile worker down or missing body_wrap"

echo "==> Auth emulator ${AUTH_HOST}"
auth_code="$(
  curl -s -o /dev/null -w '%{http_code}' -X POST \
    "http://${AUTH_HOST}/identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key=fake-api-key" \
    -H 'Content-Type: application/json' \
    -d '{"email":"preflight@terrarium.dev","password":"preflight","returnSecureToken":true}'
)"
[[ "$auth_code" != "000" ]] || fail "auth emulator identity API down at ${AUTH_HOST}"

echo "==> UI ${UI}"
curl -sf "${UI}" >/dev/null || fail "Vite UI down at ${UI}"

echo ""
echo "Ready: ${UI}"
echo "QA bridge: window.__TERRARIUM_QA__.getState()"
