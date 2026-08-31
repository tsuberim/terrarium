#!/usr/bin/env bash
# Deploy creature.wasm to Terrarium (Replit Run, or local after zig build).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ -f .env ]]; then
  set -a
  # shellcheck disable=SC1091
  source .env
  set +a
fi

WASM="$ROOT/zig-out/bin/creature.wasm"

: "${TERRARIUM_API_BASE:?Set TERRARIUM_API_BASE, e.g. https://terrarium-506917.web.app/api}"
: "${TERRARIUM_API_KEY:?Mint an API key in-game (Keys) or POST /v1/api-keys}"
: "${TERRARIUM_X:?Target cell x}"
: "${TERRARIUM_Y:?Target cell y}"

ENERGY="${TERRARIUM_ENERGY:-10000000}"

if [[ ! -f "$WASM" ]]; then
  echo "Missing $WASM — run: zig build -Doptimize=ReleaseSmall" >&2
  exit 1
fi

B64="$(base64 -w0 "$WASM" 2>/dev/null || base64 "$WASM" | tr -d '\n')"

curl -fsS -X POST "${TERRARIUM_API_BASE%/}/v1/deploy" \
  -H "Authorization: Bearer ${TERRARIUM_API_KEY}" \
  -H "Content-Type: application/json" \
  -d "$(jq -n \
    --arg b64 "$B64" \
    --argjson x "$TERRARIUM_X" \
    --argjson y "$TERRARIUM_Y" \
    --argjson energy "$ENERGY" \
    '{x:$x,y:$y,code:"// creature.wasm",energy:$energy,wasm_b64:$b64}')"

echo
