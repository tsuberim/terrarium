#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

[[ -f .env ]] && set -a && source .env && set +a
: "${FIREBASE_PROJECT_ID:?FIREBASE_PROJECT_ID required}"

if ! command -v firebase >/dev/null 2>&1; then
  echo "firebase-tools missing — run: npm install -g firebase-tools" >&2
  exit 1
fi

if lsof -ti "tcp:9099" -sTCP:LISTEN >/dev/null 2>&1; then
  echo "auth emulator: port 9099 already in use" >&2
  exit 1
fi

echo "Auth emulator: http://127.0.0.1:9099 (UI http://127.0.0.1:4000/auth)"
exec firebase emulators:start --only auth --project "$FIREBASE_PROJECT_ID"
