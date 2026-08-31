#!/usr/bin/env bash
# Replit Run button: build WASM, deploy when .env / Secrets are set.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ -f .env ]]; then
  set -a
  # shellcheck disable=SC1091
  source .env
  set +a
fi

echo "→ zig build -Doptimize=ReleaseSmall"
zig build -Doptimize=ReleaseSmall

if [[ -z "${TERRARIUM_API_KEY:-}" ]]; then
  cat <<'EOF'

Built zig-out/bin/creature.wasm

To deploy to Terrarium, paste the .env from the game (Code → Copy .env)
into sdk/zig/.env, add your API key from Keys, then press Run again.

EOF
  exit 0
fi

./scripts/deploy.sh
echo "Creature deployed at (${TERRARIUM_X}, ${TERRARIUM_Y}) — refresh Terrarium."
