#!/usr/bin/env bash
# Inject deploy-time URLs into static HTML meta tags before Firebase Hosting deploy.
set -euo pipefail

SKIN_INDEX="${1:-apps/skin/index.html}"
DASH_INDEX="${2:-apps/dashboard/index.html}"
SITE_INDEX="${3:-apps/site/index.html}"
SITE_ABOUT="${4:-apps/site/about.html}"
WS_HOST="${TERRARIUM_WS_HOST:-}"
API_URL="${TERRARIUM_API_URL:-}"
PLAY_URL="${TERRARIUM_PLAY_URL:-}"
SITE_URL="${TERRARIUM_SITE_URL:-}"
CONSOLE_URL="${TERRARIUM_CONSOLE_URL:-}"
ENV_NAME="${TERRARIUM_ENV:-staging}"

patch_meta() {
  local file="$1"
  local name="$2"
  local value="$3"
  local required="${4:-1}"
  python3 - "$file" "$name" "$value" "$required" <<'PY'
import sys
from pathlib import Path

path, name, value, required = sys.argv[1:5]
text = Path(path).read_text()
needle = f'<meta name="{name}" content="'
start = text.find(needle)
if start == -1:
    if required == "0":
        sys.exit(0)
    sys.exit(f"missing meta {name} in {path}")
start += len(needle)
end = text.find('"', start)
Path(path).write_text(text[:start] + value + text[end:])
PY
}

for file in "$SKIN_INDEX" "$DASH_INDEX"; do
  patch_meta "$file" "terrarium-env" "$ENV_NAME"
done

patch_meta "$SKIN_INDEX" "terrarium-ws-host" "$WS_HOST" 0
patch_meta "$DASH_INDEX" "terrarium-api" "$API_URL" 0

if [[ -n "$PLAY_URL" ]]; then
  for file in "$SKIN_INDEX" "$DASH_INDEX" "$SITE_INDEX" "$SITE_ABOUT"; do
    [[ -f "$file" ]] && patch_meta "$file" "terrarium-play" "$PLAY_URL" 0
  done
fi

if [[ -n "$SITE_URL" ]]; then
  for file in "$SKIN_INDEX" "$DASH_INDEX" "$SITE_INDEX" "$SITE_ABOUT"; do
    [[ -f "$file" ]] && patch_meta "$file" "terrarium-home" "$SITE_URL" 0
  done
  for file in "$SKIN_INDEX" "$DASH_INDEX" "$SITE_INDEX" "$SITE_ABOUT"; do
    [[ -f "$file" ]] && patch_meta "$file" "terrarium-about" "${SITE_URL%/}/about.html" 0
  done
fi

if [[ -n "$CONSOLE_URL" ]]; then
  for file in "$SKIN_INDEX" "$DASH_INDEX" "$SITE_INDEX" "$SITE_ABOUT"; do
    [[ -f "$file" ]] && patch_meta "$file" "terrarium-console" "$CONSOLE_URL" 0
  done
fi

echo "patched ws host: ${WS_HOST:-<same-origin>}"
echo "patched api: ${API_URL:-<fallback>}"
echo "patched play: ${PLAY_URL:-<unchanged>}"
