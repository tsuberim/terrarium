#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> Rust tests"
cargo test --workspace

echo "==> Server release build"
cargo build --release -p terrarium-server

if [[ -f apps/skin/package-lock.json ]]; then
  echo "==> Frontend typecheck + build"
  (cd apps/skin && npm ci --silent && npm run build)
fi

echo "OK"
