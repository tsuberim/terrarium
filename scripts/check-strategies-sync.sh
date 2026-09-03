#!/usr/bin/env bash
# Fail if strategy Rust was edited without syncing WAT into examples.rs.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
before="$(git hash-object crates/sim/src/examples.rs 2>/dev/null || echo none)"
"$(dirname "$0")/build-strategies.sh" >/dev/null
after="$(git hash-object crates/sim/src/examples.rs)"
if [[ "$before" == "$after" ]]; then
  echo "strategies/examples.rs in sync"
  exit 0
fi
echo "examples.rs out of sync — run ./scripts/build-strategies.sh and commit" >&2
git diff --stat crates/sim/src/examples.rs >&2 || true
exit 1
