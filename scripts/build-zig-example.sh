#!/usr/bin/env bash
# Build sdk/zig example to wasm32. Dev-only — CI does not require Zig.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/sdk/zig"

if ! command -v zig >/dev/null; then
  echo "Install Zig: https://ziglang.org/download/" >&2
  exit 1
fi

zig build -Doptimize=ReleaseSmall
OUT="$ROOT/sdk/zig/zig-out/bin/creature.wasm"
echo "Built $OUT"
echo "Upload in Terrarium → Programs, or: open $OUT"
