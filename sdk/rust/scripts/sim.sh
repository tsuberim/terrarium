#!/usr/bin/env bash
# Run sandbox preview against local or remote API (requires auth / API key via server).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SDK="$(cd "$(dirname "$0")/.." && pwd)"

if [[ -f "$SDK/.env" ]]; then
  set -a
  # shellcheck disable=SC1091
  source "$SDK/.env"
  set +a
fi

: "${TERRARIUM_API_BASE:?Set TERRARIUM_API_BASE}"

SCENARIO="${TERRARIUM_SCENARIO:-open}"
TICKS="${TERRARIUM_TICKS:-100}"
SOURCE_FILE="${1:-$ROOT/services/compile-worker/template/src/user.rs}"

if [[ ! -f "$SOURCE_FILE" ]]; then
  echo "Missing source: $SOURCE_FILE" >&2
  exit 1
fi

SOURCE="$(cat "$SOURCE_FILE")"

COMPILE_URL="${TERRARIUM_API_BASE%/}/v1/compile"
SANDBOX_URL="${TERRARIUM_API_BASE%/}/v1/sandbox/run"

AUTH=()
if [[ -n "${TERRARIUM_API_KEY:-}" ]]; then
  AUTH=(-H "Authorization: Bearer ${TERRARIUM_API_KEY}")
elif [[ -n "${FIREBASE_ID_TOKEN:-}" ]]; then
  AUTH=(-H "Authorization: Bearer ${FIREBASE_ID_TOKEN}")
fi

COMPILED="$(jq -n --arg src "$SOURCE" '{language:"rust",source:$src}' | \
  curl -fsS "${AUTH[@]}" -X POST "$COMPILE_URL" -H 'Content-Type: application/json' -d @-)"

OK="$(echo "$COMPILED" | jq -r '.ok')"
if [[ "$OK" != "true" ]]; then
  echo "$COMPILED" | jq '.diagnostics'
  exit 1
fi

WASM_B64="$(echo "$COMPILED" | jq -r '.wasm_b64')"

curl -fsS "${AUTH[@]}" -X POST "$SANDBOX_URL" -H 'Content-Type: application/json' \
  -d "$(jq -n --arg b64 "$WASM_B64" --arg sc "$SCENARIO" --argjson t "$TICKS" \
    '{wasm_b64:$b64,scenario:$sc,ticks:$t}')" | jq '{alive, ticks_run, death_reason, bench}'
