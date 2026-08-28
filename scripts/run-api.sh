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
export TERRARIUM_LISTEN="${TERRARIUM_LISTEN:-127.0.0.1:3000}"
export TERRARIUM_DB="${TERRARIUM_DB:-./terrarium.db}"
export TERRARIUM_DASHBOARD_DIR="${TERRARIUM_DASHBOARD_DIR:-apps/dashboard}"

exec cargo run --manifest-path crates/api/Cargo.toml --release
