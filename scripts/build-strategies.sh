#!/usr/bin/env bash
# Dev-only: compile Rust strategies to WASM and sync WAT into example programs.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/strategies"
export CARGO_TARGET_DIR="$ROOT/strategies/target"

rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true

cargo build --release -p strategy-predator --target wasm32-unknown-unknown
cargo build --release -p strategy-scavenger --target wasm32-unknown-unknown
cargo build --release -p strategy-prey --target wasm32-unknown-unknown
cargo build --release -p strategy-hawk --target wasm32-unknown-unknown
cargo run -p strategy-tools --bin sync-examples

echo "Done. Commit updated examples.rs + examples.ts if strategy logic changed."
