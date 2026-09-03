#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
if diff -q "$ROOT/crates/server/src/openapi.json" "$ROOT/docs/public/openapi.json" >/dev/null; then
  echo "openapi.json in sync"
  exit 0
fi
echo "openapi.json drift: run ./scripts/sync-openapi.sh" >&2
diff -u "$ROOT/crates/server/src/openapi.json" "$ROOT/docs/public/openapi.json" | head -40 >&2 || true
exit 1
