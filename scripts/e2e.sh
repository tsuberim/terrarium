#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
"$(dirname "$0")/stack-preflight.sh"
npm run e2e --prefix apps/skin
