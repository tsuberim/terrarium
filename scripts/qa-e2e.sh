#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
"$(dirname "$0")/qa-preflight.sh"
npm run qa:e2e --prefix apps/skin
