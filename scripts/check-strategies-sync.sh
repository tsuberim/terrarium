#!/usr/bin/env bash
# Verify strategy crates compile to wasm32 (WAT sync is dev-only; wasmprinter output varies by OS).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/strategies"
export CARGO_TARGET_DIR="$ROOT/strategies/target"

rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
cargo build --release -p strategy-predator --target wasm32-unknown-unknown -q
cargo build --release -p strategy-scavenger --target wasm32-unknown-unknown -q
cargo build --release -p strategy-prey --target wasm32-unknown-unknown -q
cargo build --release -p strategy-hawk --target wasm32-unknown-unknown -q
echo "strategies compile ok"
