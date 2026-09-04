# Tech debt

Known gaps, shortcuts, and cleanup work. **Not** product decisions — those live in [../product/requirements.md](../product/requirements.md) §16 and [../product/vision.md](../product/vision.md).

**When you pay down debt:** update this file in the same PR. When debt becomes a product choice, move it to the PRD and link back here.

---

## Priority legend

| Tag | Meaning |
|-----|---------|
| **P0** | Blocks prod reliability or misleads users/agents |
| **P1** | High friction for dev, deploy, or scale |
| **P2** | Cleanup, consistency, nice-to-have |

---

## Infrastructure & runtime

| ID | Priority | Debt | Impact | Direction |
|----|----------|------|--------|-----------|
| TD-INF-1 | **P0** | Prod DB is ephemeral SQLite on Cloud Run — world resets on redeploy / instance recycle | Players lose progress; prod is a demo | Persistent volume (Cloud SQL / file-backed volume) or accept demo-only prod |
| TD-INF-2 | **P1** | Cloud Run `min-instances=0` — cold start 15–45s | First visitor waits; smoke tests retry | Raise min-instances when traffic justifies cost, or warm-up cron |
| TD-INF-3 | **P1** | No staging environment | Every change hits prod | Second Firebase project + Cloud Run service (explicit non-goal in PRD until decided) |
| TD-INF-4 | **P1** | Compile worker optional in prod; may not be always-on | Studio compile fails if worker down | Deploy `terrarium-compile` to Cloud Run; wire `COMPILE_WORKER_URL` on API |
| TD-INF-5 | **P2** | WebSocket bypasses Firebase Hosting (`VITE_WS_BASE` required) | Easy to ship broken prod WS if config skipped | CI gate already fails missing WS base; document in deploy checklist only |
| TD-INF-6 | **P2** | JSON wire encoding per client | CPU/bandwidth at ~100+ spectators | Binary codec (MessagePack/CBOR) per [architecture.md](architecture.md) |

---

## Simulation & server

| ID | Priority | Debt | Impact | Direction |
|----|----------|------|--------|-----------|
| TD-SIM-1 | **P1** | No spatial hash for creature collision — linear scans | Tick overrun as entity count grows | Spatial hash in sim; see architecture Phase A remainder |
| TD-SIM-2 | **P1** | No per-tick time budget / fair round-robin when overrun | World clock slips under load | `TICK_BUDGET_US` + cursor (architecture Phase B) |
| TD-SIM-3 | **P2** | No chunking / viewport subscription | Bandwidth ∝ world size, not camera | Phase C in architecture |
| TD-SIM-4 | **P2** | Energy ledger not exposed on public wire | External tools can't audit deflation | Internal-only by design; revisit if we ship analytics |
| TD-SIM-5 | **P2** | Food/node tuning constants TBD | Economy balance unproven at scale | energy-budget.md constants table |
| TD-SIM-6 | **P2** | Local corrupt SQLite → `UNIQUE constraint failed: creatures.x,y` | Dev confusion | Document reset; optional startup integrity check |
| TD-SIM-7 | **P1** | ABI v2 **u64** creature ids in sim; server wire/DB still UUID strings | External control + ABI docs ahead of server | **Done** — migration 011 + wire string encoding |
| TD-SIM-8 | **P1** | Deploy does not set sim `owner_id` / Init on place | Spawn owner chain broken for deployed creatures | **Done** — deploy sets `owner_id = id` |
| TD-SIM-9 | **P2** | Creature inbox not persisted | Signals lost on restart | Accept for v1 or persist inbox in checkpoint |
| TD-EXT-1 | **P1** | No control WS or sim inject/fan-out bridge | External signal/recv (PRD §13) blocked | **Done** — `/v1/control/ws` + sim bridge |

---

## Frontend & product surface

| ID | Priority | Debt | Impact | Direction |
|----|----------|------|--------|-----------|
| TD-UI-1 | **P2** | Mintlify Studio + Rust SDK page | Player onboarding | **Done** — keep synced on UX changes |
| TD-UI-2 | **P1** | Deploy UX: modal centered; must pick cell before opening deploy | QA/agent friction; easy user mistake | PRD DEP-10 documents workaround; consider inline deploy bar |
| TD-UI-3 | **P1** | Studio closed → `pointer-events: none` on shell | Clicks silently fail | Open-by-default for signed-in, or visible disabled state |
| TD-UI-4 | **P2** | No visual regression / screenshot QA | Layout regressions caught late | Playwright screenshots (out of scope v1 per qa/README) |
| TD-UI-5 | **P2** | Dev panel (sim config) is dev-only with no prod guard beyond build | Accidental exposure if misconfigured | Keep behind `DEV_MODE`; audit env in prod deploy |
| TD-UI-6 | **P2** | `api.ts` `WorldEvent.signal` still documents legacy `byte` field | Type drift vs sim (metadata-only signal events) | Align with server `wire.rs` when id migration lands |

---

## Documentation

| ID | Priority | Debt | Impact | Direction |
|----|----------|------|--------|-----------|
| TD-DOC-1 | **P2** | `docs/internal/` folder split | Was flat root | **Done** — see [../../README.md](../../README.md) |
| TD-DOC-7 | **P2** | Doc map | Hard to discover | **Done** — [docs/README.md](../README.md) |
| TD-DOC-9 | **P2** | Mintlify dashboard may still point at old `docs/` root | Public site breaks on deploy | Set Mintlify GitHub path to **`docs/public`** |
| TD-DOC-2 | **P2** | [sim/host-abi.md](sim/host-abi.md) reframed | Was WAT-first | Done |
| TD-DOC-5 | **P2** | [testing.md](testing.md) stale refs | Fixed | **Done** |
| TD-DOC-6 | **P2** | `authoring.md` removed | Was stub | **Done** — PRD §9 + sdk READMEs |
| TD-DOC-4 | **P2** | workflow / ops / engineering split | Was one god doc | **Done** |
| TD-DOC-3 | **P2** | vision trimmed | Overlapped PRD | **Done** |
| TD-DOC-8 | **P2** | Open product questions scattered (PRD §16, vision, energy-budget) | Decisions get re-litigated | Keep PRD canonical; link from here only for **engineering blockers** |

---

## Testing & QA

| ID | Priority | Debt | Impact | Direction |
|----|----------|------|--------|-----------|
| TD-QA-1 | **P2** | Playwright runs `docs/internal/qa/scenarios/*.yaml` | Was duplicated | **Done** |
| TD-QA-2 | **P2** | `signed-out-gate.yaml` in Playwright suite | Auth gate regression | **Done** |
| TD-QA-3 | **P2** | API smoke deploy picks cell heuristically | May flake on crowded worlds | Pin `QA_DEPLOY_X/Y` (default 32,32) + scan fallback — **Done** |
| TD-QA-4 | **P2** | browser-qa skill stale text | Agent confusion | **Done** |
| TD-QA-5 | **P2** | No `/api/v1/qa/state` HTTP endpoint | Agents rely on browser evaluate only | Window bridge sufficient for v1 |

---

## Code & repo hygiene

| ID | Priority | Debt | Impact | Direction |
|----|----------|------|--------|-----------|
| TD-CODE-1 | **P2** | `strategies/` → WAT sync is manual (`build-strategies.sh`) | Drift if someone edits Rust without sync | **Done** — removed; no bundled examples |
| TD-CODE-2 | **P2** | OpenAPI deploy `code` field clarified | Secondary path unclear | **Done** |
| TD-CODE-3 | **P2** | Two SDKs (Rust in-game, Zig external) | Story split | **Done** — Rust SDK + Studio primary; WASM upload + API keys for external |
| TD-CODE-4 | **P2** | Uncommitted large feature branch (Studio, QA, auth emulator) | Main may lag docs | Land PR; docs already ahead of main |

---

## Payments & economy (engineering blockers)

Product direction open in PRD §16. Engineering debt once product decides:

| ID | Priority | Debt | Notes |
|----|----------|------|-------|
| TD-PAY-1 | **P1** | No payment provider integration | Faucet stands in for credits |
| TD-PAY-2 | **P2** | No cash-out / ledger reconciliation | Economy is in-world only |

---

## Suggested paydown order

1. **TD-INF-1** — persistent prod DB (when prod should be real)
2. **TD-SIM-7 + TD-EXT-1** — u64 ids + control WS (before external control)
3. **TD-INF-4** — prod compile worker always-on
4. **TD-SIM-1 + TD-SIM-2** — before pushing entity count
5. **TD-QA-3** — pin deploy cell in API smoke — **Done**

---

## Related docs

| Doc | Relationship |
|-----|--------------|
| [../product/requirements.md](../product/requirements.md) | Shipped behavior + open product questions |
| [../product/vision.md](../product/vision.md) | Principles |
| [architecture.md](architecture.md) | Scale path |
| [../workflow/README.md](../workflow/README.md) | How to dev/test |
| [../qa/README.md](../qa/README.md) | QA framework |
