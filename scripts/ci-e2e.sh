#!/usr/bin/env bash
# CI: Playwright e2e against full dev stack.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=ci-stack.sh
source "$ROOT/scripts/ci-stack.sh"

ci_stack_write_env true
ci_stack_build
ci_stack_start_api
ci_stack_install_playwright
ci_stack_start_ui
ci_stack_wait true

echo "==> Playwright e2e"
npm run e2e --prefix apps/skin

echo "CI e2e passed."
