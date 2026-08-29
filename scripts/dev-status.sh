#!/usr/bin/env bash
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
web=$(curl -sf http://127.0.0.1:5173/ >/dev/null && echo up || echo down)
api=$(curl -sf http://127.0.0.1:8080/health 2>/dev/null || echo down)
echo "frontend :5173 → $web"
echo "api      :8080 → $api"
lsof -i :5173 -i :8080 -P -n 2>/dev/null | grep LISTEN || true
