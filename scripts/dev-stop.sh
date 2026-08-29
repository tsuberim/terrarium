#!/usr/bin/env bash
for port in 8080 5173; do
  pids=$(lsof -ti "tcp:${port}" -sTCP:LISTEN 2>/dev/null || true)
  [[ -n "$pids" ]] && kill $pids 2>/dev/null || true
done
echo "Stopped."
