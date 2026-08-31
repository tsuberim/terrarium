#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> Rust fmt + clippy + tests"
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

echo "==> Server release build"
cargo build --release -p terrarium-server

if [[ -f apps/skin/package-lock.json ]]; then
  echo "==> Frontend lint + typecheck + build"
  (cd apps/skin && npm ci --silent && npm run lint && npm run build)
fi

echo "OK"
