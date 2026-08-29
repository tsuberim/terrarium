#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

kill_port() {
  local port=$1
  local pids
  pids=$(lsof -ti "tcp:${port}" -sTCP:LISTEN 2>/dev/null || true)
  [[ -n "$pids" ]] && kill $pids 2>/dev/null || true
}

kill_port 8080
kill_port 5173

rm -f "$ROOT/.dev/api.pid" "$ROOT/.dev/web.pid"
echo "Dev servers stopped."
