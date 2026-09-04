#!/usr/bin/env bash
# Sync main branch protection required checks with docs/internal/workflow/ci.md.
set -euo pipefail

REPO="${1:-$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || echo tsuberim/terrarium)}"

echo "==> Required checks on ${REPO}@main"
gh api "repos/${REPO}/branches/main/protection/required_status_checks" \
  -X PATCH \
  --input - <<EOF
{
  "strict": true,
  "contexts": [
    "test / rust",
    "test / frontend",
    "test / docker",
    "test / smoke",
    "test / e2e"
  ]
}
EOF

echo "Done. Auto-merge now waits for smoke and e2e."
