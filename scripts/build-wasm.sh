#!/usr/bin/env bash
# Build crates/kernel to WASM (optional — tests / later cell guests).
# The live sim host is native (`crates/host`); this is not the deploy path.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/apps/skin/pkg"

cd "$ROOT"
cargo build -p terrarium-kernel --features wasm --target wasm32-unknown-unknown --release
wasm-bindgen \
  --target web \
  --out-dir "$OUT" \
  "$ROOT/target/wasm32-unknown-unknown/release/terrarium_kernel.wasm"
echo "wrote $OUT"
