#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

if [[ -f .env ]]; then
  set -a
  # shellcheck disable=SC1091
  source .env
  set +a
fi

export TERRARIUM_ENV="${TERRARIUM_ENV:-local}"
export PORT="${PORT:-8080}"
export TERRARIUM_HOST_TOKEN="${TERRARIUM_HOST_TOKEN:-}"
export SKIN_DIR="${SKIN_DIR:-apps/skin}"

exec cargo run --manifest-path crates/host/Cargo.toml --release
