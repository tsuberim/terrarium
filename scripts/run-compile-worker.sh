#!/usr/bin/env bash
# Run the compile worker locally (requires wasm32-unknown-unknown).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/services/compile-worker"

rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true

export TEMPLATE_DIR="$ROOT/services/compile-worker/template"
export SDK_PATH="$ROOT/sdk/rust/terrarium-sdk"
export PORT="${COMPILE_PORT:-8081}"
export RUST_LOG="${RUST_LOG:-info}"

exec cargo run --release
