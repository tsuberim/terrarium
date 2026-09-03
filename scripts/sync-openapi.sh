#!/usr/bin/env bash
# Copy server OpenAPI spec to Mintlify public docs (single source of truth).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cp "$ROOT/crates/server/src/openapi.json" "$ROOT/docs/public/openapi.json"
echo "Synced docs/public/openapi.json from crates/server/src/openapi.json"
