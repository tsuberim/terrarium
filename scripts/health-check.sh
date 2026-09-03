#!/usr/bin/env bash
# Shared stack health probes — sourced by stack-preflight.sh and api-smoke.sh.
set -euo pipefail

health_check_api() {
  local api="${1:?}"
  curl -sf "${api}/health" | python3 -c "
import json, sys
d = json.load(sys.stdin)
assert d.get('status') == 'ok', d
"
}

health_check_compile() {
  local compile="${1:?}"
  curl -sf "${compile}/health" | python3 -c "
import json, sys
d = json.load(sys.stdin)
assert d.get('status') == 'ok' and d.get('body_wrap') is True, d
"
}

health_check_auth_emulator() {
  local auth_host="${1:?}"
  local code
  code="$(
    curl -s -o /dev/null -w '%{http_code}' -X POST \
      "http://${auth_host}/identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key=fake-api-key" \
      -H 'Content-Type: application/json' \
      -d '{"email":"health@terrarium.dev","password":"health","returnSecureToken":true}'
  )"
  [[ "$code" != "000" ]]
}
