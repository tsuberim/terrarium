#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

"$ROOT/scripts/dev-stop.sh" >/dev/null 2>&1 || true

if [[ -f .env ]]; then
  set -a
  # shellcheck disable=SC1091
  source .env
  set +a
fi

: "${FIREBASE_PROJECT_ID:?Run ./scripts/setup-dev.sh first}"
[[ -f apps/skin/.env.local ]] || { echo "Run ./scripts/setup-dev.sh"; exit 1; }
command -v cargo-watch >/dev/null 2>&1 || { echo "Run ./scripts/setup-dev.sh"; exit 1; }

mkdir -p "$ROOT/.dev/logs"
export LISTEN_ADDR="${LISTEN_ADDR:-0.0.0.0:8080}"
export DATABASE_URL="${DATABASE_URL:-sqlite::memory:}"
export FAUCET_ENABLED="${FAUCET_ENABLED:-true}"

: >"$ROOT/.dev/logs/api.log"
: >"$ROOT/.dev/logs/web.log"

nohup cargo watch -q -x 'run -p terrarium-server --bin terrarium-server' \
  >>"$ROOT/.dev/logs/api.log" 2>&1 &
echo $! >"$ROOT/.dev/api.pid"

nohup env -C "$ROOT/apps/skin" ./node_modules/.bin/vite \
  >>"$ROOT/.dev/logs/web.log" 2>&1 &
echo $! >"$ROOT/.dev/web.pid"

for _ in $(seq 1 20); do
  if curl -sf "http://127.0.0.1:5173/" >/dev/null 2>&1 \
    && curl -sf "http://127.0.0.1:8080/health" >/dev/null 2>&1; then
    echo "http://127.0.0.1:5173"
    echo "logs: .dev/logs/{api,web}.log"
    exit 0
  fi
  sleep 1
done

echo "Dev server failed to start:"
tail -5 "$ROOT/.dev/logs/web.log" 2>/dev/null || true
tail -5 "$ROOT/.dev/logs/api.log" 2>/dev/null || true
exit 1
