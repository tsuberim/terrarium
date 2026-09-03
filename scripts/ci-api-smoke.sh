#!/usr/bin/env bash
# CI: API smoke against live server + compile worker + auth emulator.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=ci-stack.sh
source "$ROOT/scripts/ci-stack.sh"

ci_stack_write_env false
ci_stack_build
ci_stack_start_api
ci_stack_wait false

echo "==> API smoke"
./scripts/api-smoke.sh

echo "CI API smoke passed."
