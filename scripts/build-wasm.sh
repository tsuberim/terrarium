#!/usr/bin/env bash
# Build crates/kernel to WASM and emit apps/skin/pkg/ for the static skin.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="$ROOT/crates/kernel/Cargo.toml"
OUT="$ROOT/apps/skin/pkg"

cd "$ROOT"
cargo build --manifest-path "$MANIFEST" --target wasm32-unknown-unknown --release
wasm-bindgen \
  --target web \
  --out-dir "$OUT" \
  "$ROOT/crates/kernel/target/wasm32-unknown-unknown/release/terrarium_kernel.wasm"
echo "wrote $OUT"
