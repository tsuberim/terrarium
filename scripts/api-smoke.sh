#!/usr/bin/env bash
# Headless API smoke: auth emulator → compile → sandbox → (optional) deploy.
# Requires server :8080, compile-worker :8081, auth emulator :9099.
set -euo pipefail

API="${API_BASE:-http://127.0.0.1:8080/api}"
AUTH_HOST="${FIREBASE_AUTH_EMULATOR_HOST:-127.0.0.1:9099}"
COMPILE="${COMPILE_WORKER_URL:-http://127.0.0.1:8081}"
SMOKE_EMAIL="${SMOKE_EMAIL:-qa@terrarium.dev}"
SMOKE_PASSWORD="${SMOKE_PASSWORD:-qa-terrarium}"
SMOKE_DEPLOY="${SMOKE_DEPLOY:-1}"
SMOKE_DEPLOY_X="${SMOKE_DEPLOY_X:-32}"
SMOKE_DEPLOY_Y="${SMOKE_DEPLOY_Y:-32}"

json_get() {
  python3 -c "import json,sys; print(json.load(sys.stdin)$1)" 2>/dev/null
}

auth_emulator_token() {
  local email="$1" password="$2"
  local base="http://${AUTH_HOST}/identitytoolkit.googleapis.com/v1/accounts"
  local token
  token="$(
    curl -sf -X POST "${base}:signInWithPassword?key=fake-api-key" \
      -H 'Content-Type: application/json' \
      -d "{\"email\":\"${email}\",\"password\":\"${password}\",\"returnSecureToken\":true}" \
    | json_get "['idToken']" || true
  )"
  if [[ -n "$token" ]]; then
    echo "$token"
    return 0
  fi
  curl -sf -X POST "${base}:signUp?key=fake-api-key" \
    -H 'Content-Type: application/json' \
    -d "{\"email\":\"${email}\",\"password\":\"${password}\",\"returnSecureToken\":true}" \
  | json_get "['idToken']"
}

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=health-check.sh
source "$ROOT/scripts/health-check.sh"

echo "==> Server health"
health_check_api "$API"

echo "==> Compile worker (body_wrap)"
health_check_compile "$COMPILE"
echo "body_wrap ok"

echo "==> Auth emulator sign-in"
TOKEN="$(auth_emulator_token "${SMOKE_EMAIL}" "${SMOKE_PASSWORD}")"
AUTH=(-H "Authorization: Bearer ${TOKEN}")

echo "==> GET /v1/me"
ME="$(curl -sf "${AUTH[@]}" "${API}/v1/me")"
echo "$ME" | python3 -c "import json,sys; d=json.load(sys.stdin); assert d.get('uid'); print('uid', d['uid'], 'credits', d.get('credits'))"

echo "==> POST /v1/compile (default creature source)"
COMPILE_BODY="$(mktemp)"
SOURCE_FILE="$(mktemp)"
TESTS_FILE="$(mktemp)"
cat >"$SOURCE_FILE" <<'EOF'
move_forward();
EOF
cat >"$TESTS_FILE" <<'EOF'
#[terrarium::test]
fn open_field() {
    run_ticks(20);
    assert!(alive());
}
EOF
python3 -c "import json, pathlib; print(json.dumps({'language':'rust','source': pathlib.Path('$SOURCE_FILE').read_text(), 'tests': pathlib.Path('$TESTS_FILE').read_text()}))" >"$COMPILE_BODY"
rm -f "$SOURCE_FILE" "$TESTS_FILE"
COMPILE_RESP="$(curl -sf "${AUTH[@]}" -H 'Content-Type: application/json' -d @"$COMPILE_BODY" "${API}/v1/compile")"
rm -f "$COMPILE_BODY"
WASM="$(echo "$COMPILE_RESP" | json_get "['wasm_b64']")"
echo "$COMPILE_RESP" | python3 -c "
import json, sys, base64
d = json.load(sys.stdin)
assert d.get('ok') is True, d
w = base64.b64decode(d['wasm_b64'])
assert w[:4] == b'\\x00asm', w[:8]
print('wasm ok:', len(w), 'bytes')
"

echo "==> POST /v1/sandbox/run (open_field test)"
SANDBOX_BODY="$(mktemp)"
python3 -c "import json; print(json.dumps({'wasm_b64':'${WASM}','test':{'name':'open_field','ticks':20,'facing':0,'start_energy':4000000,'tiles':[],'assertions':[{'Alive':{'expected':True,'line':1}}]}}))" >"$SANDBOX_BODY"
SANDBOX="$(curl -sf "${AUTH[@]}" -H 'Content-Type: application/json' -d @"$SANDBOX_BODY" "${API}/v1/sandbox/run")"
rm -f "$SANDBOX_BODY"
echo "$SANDBOX" | python3 -c "
import json, sys
d = json.load(sys.stdin)
assert d.get('ok') is True, d
frames = d.get('frames') or []
assert len(frames) >= 1, d
assert d.get('test_passed') is True, d
print('sandbox ok:', len(frames), 'frames, alive=', d.get('alive'))
"

if [[ "$SMOKE_DEPLOY" == "1" ]]; then
  echo "==> POST /v1/deploy"
  WORLD="$(curl -sf "${API}/v1/world")"
  ME="$(curl -sf "${AUTH[@]}" "${API}/v1/me")"
  read -r DEPLOY_X DEPLOY_Y DEPLOY_ENERGY FAUCET_AMOUNT <<<"$(python3 -c "
import json, os
world = json.loads('''$WORLD''')
me = json.loads('''$ME''')
deploy_cost = int(world['deploy_cost'])
corpse = int(world['corpse_energy'])
need = deploy_cost + corpse
credits = int(me.get('credits') or 0)
faucet = max(0, need - credits)
occupied = {(int(c['x']), int(c['y'])) for c in world.get('creatures') or []}
solid = {(int(t['x']), int(t['y'])) for t in world.get('tiles') or [] if t.get('kind') == 1}
pin_x = int('${SMOKE_DEPLOY_X}')
pin_y = int('${SMOKE_DEPLOY_Y}')
x = y = None
if (pin_x, pin_y) not in occupied and (pin_x, pin_y) not in solid:
    x, y = pin_x, pin_y
else:
    for yy in range(0, 64):
        for xx in range(0, 64):
            if (xx, yy) not in occupied and (xx, yy) not in solid:
                x, y = xx, yy
                break
        if x is not None:
            break
if x is None:
    raise SystemExit('no deployable cell found')
print(x, y, deploy_cost, faucet)
")"
  if [[ "$FAUCET_AMOUNT" -gt 0 ]]; then
    FAUCET_CHUNK="${FAUCET_CHUNK:-10000000}"
    remaining="$FAUCET_AMOUNT"
    while [[ "$remaining" -gt 0 ]]; do
      amount="$remaining"
      if [[ "$amount" -gt "$FAUCET_CHUNK" ]]; then
        amount="$FAUCET_CHUNK"
      fi
      echo "    faucet +${amount}"
      curl -sf -X POST "${AUTH[@]}" -H 'Content-Type: application/json' \
        -d "{\"amount\":${amount}}" "${API}/v1/faucet" >/dev/null
      remaining=$((remaining - amount))
    done
  fi
  DEPLOY_BODY="$(mktemp)"
  python3 -c "import json; print(json.dumps({'x':${DEPLOY_X},'y':${DEPLOY_Y},'code':'api-smoke','energy':${DEPLOY_ENERGY},'wasm_b64':'${WASM}'}))" >"$DEPLOY_BODY"
  curl -sf "${AUTH[@]}" -H 'Content-Type: application/json' -d @"$DEPLOY_BODY" "${API}/v1/deploy" | python3 -c "
import json, sys
d = json.load(sys.stdin)
assert d.get('id'), d
print('deployed', d['id'], 'at', d.get('x'), d.get('y'))
"
  rm -f "$DEPLOY_BODY"
fi

echo "API smoke passed."
