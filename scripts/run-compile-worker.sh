#!/usr/bin/env bash
# Run the compile worker locally with auto-reload (requires wasm32-unknown-unknown).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/services/compile-worker"

rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true

export TEMPLATE_DIR="$ROOT/services/compile-worker/template"
export SDK_PATH="$ROOT/sdk/rust/terrarium-sdk"
export PORT="${COMPILE_PORT:-8081}"
export RUST_LOG="${RUST_LOG:-info}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

if lsof -ti "tcp:${PORT}" -sTCP:LISTEN >/dev/null 2>&1; then
  echo "compile-worker: port ${PORT} already in use — run ./scripts/dev-stop.sh" >&2
  exit 1
fi

exec cargo watch -q --delay 2 \
  -w src \
  -w template \
  -w "$ROOT/sdk/rust/terrarium-sdk" \
  -w "$ROOT/crates/test-spec" \
  -x 'run --release --quiet'
