#!/usr/bin/env bash
for port in 8080 8081 5173 9099 4000; do
  pids=$(lsof -ti "tcp:${port}" -sTCP:LISTEN 2>/dev/null || true)
  [[ -n "$pids" ]] && kill $pids 2>/dev/null || true
done
echo "Stopped."
