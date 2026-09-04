# Terrarium WASM host ABI (v2)

Creatures are WASM modules importing `"terrarium"`. **No legacy v1 surface.**

> Player docs: [host-abi.mdx](../../../public/reference/host-abi.mdx). PRD §5–11.

## Design goals

1. **Simple** — one envelope shape, one memory page, three imports.
2. **Extensible** — new action kinds and state fields without breaking old WASM.
3. **Symmetric** — host→guest and guest→host both use fixed memory slots (no pointer args, no 9-arg syscalls).
4. **Efficient** — fixed 64 B slots (one cache line), three imports, guest writes Action once per tick; no cross-boundary pointer marshalling or per-field syscalls.

## Module contract

| Export | Signature | Role |
|--------|-----------|------|
| `memory` | linear memory ≥ 8 KiB | Required |
| `main` | `() -> ()` | Lifetime fiber; resumed each tick |

## Imports (only three)

| Import | Signature | Effect |
|--------|-----------|--------|
| `act` | `() -> i32` | Read guest-written **Action** slot; queue world action; async yield on success |
| `recv` | `() -> i32` | Pop inbox into host-written **Inbox** slot; return 1 if message |
| `rand` | `() -> u64` | Pseudorandom (seed: creature id + tick) |

All reads (self, tiles, init) are **memory only**. No `sleep`, no scalar explosion.

## Slice execution

| Slice end | Effect |
|-----------|--------|
| First successful `act()` | One action; **suspend** |
| Opcode gas exhausted | **Suspend** |
| `main` returns | **Suicide** |
| Trap / bad args | Death |

---

## The envelope (64 bytes, forever)

One struct for **actions**, **signals**, **spawn args**, **birth init**, and **inbox body**.

```
offset  size   field
0       4      kind   u32   (discriminant — see tables below)
4       4      _pad   u32   (must be 0 today; future flags)
8       56     words  u64[7]
```

**64 bytes exactly.** No per-kind layouts. SDK/host interpret `words[]` by `kind`.

### Core action kinds (`kind` 1–127)

Reserved: `0` = empty/no-op. **128–255** reserved for future core. **≥256** experimental (may trap in prod).

| kind | name | words[0] | words[1] | words[2..6] |
|------|------|----------|----------|-------------|
| 1 | Move | rel | — | — |
| 2 | Rotate | delta (i32 in low bits) | — | — |
| 3 | Dig | rel | — | — |
| 4 | Place | rel | — | — |
| 5 | Eat | rel | — | — |
| 6 | Hit | rel | — | — |
| 7 | Spawn | energy | owner_id | child init (5×u64 = 40 B) |
| 8 | Signal | target_id | — | message body (5×u64) |
| 9 | Broadcast | — | — | message body (5×u64) |

**Rel** (const enum in low byte of rel word): `Fwd=0` … `FwdL=5`.

Spatial actions: invalid `rel` → action fails silently (same as blocked move).

### Messages

Inbox slot = `sender_id u64` + **envelope 64 B** (same shape; `kind` is the message opcode your program defines).

Spawn child init = **envelope** copied to **Init** slot (header zeroed except your bootstrap fields in `words`).

---

## ABI memory page

Single contiguous page at **`ABI_BASE = 4096`**. Host refreshes read regions each tick; guest writes **Action** before `act()`.

| Region | Offset | Size | Writer | Content |
|--------|--------|------|--------|---------|
| **Header** | +0 | 32 | host | magic, abi_version, layout_version, tick |
| **Self** | +32 | 64 | host | creature state (see below) |
| **RelTiles** | +96 | 288 | host | 6 × TileView |
| **Vision** | +384 | variable | host | `VisionEntry[]` (count in Self) |
| **Init** | +4096 | 64 | host once | birth envelope (deploy = zeros) |
| **Inbox** | +4160 | 72 | host on recv | sender u64 + envelope |
| **Action** | +4232 | 64 | **guest** | pending action for `act()` |

Vision grows down-page from +384; max bounded by sim `r_vis`. Header `layout_version` bumps if region sizes change.

### Header (32 B)

| Field | Type | Notes |
|-------|------|-------|
| magic | u32 | `0x5452_0002` (`TR` + version nibble) |
| abi_version | u32 | **2** — guest may trap if unsupported |
| layout_version | u32 | page layout; append-only evolution |
| tick | u64 | world tick |
| _reserved | u64[2] | zero |

### Self (64 B)

| Field | Type |
|-------|------|
| id | u64 |
| owner_id | u64 |
| pos_x, pos_y | i32 |
| facing | u32 |
| energy | i64 |
| health, max_health | i32 |
| uptime | u32 |
| inbox_len | u32 |
| vision_count | u32 |
| _reserved | u32 |

New self fields **append after `_reserved`** with `layout_version++`. Old WASM ignores tail.

### TileView (48 B)

| Field | Type |
|-------|------|
| kind | u32 |
| flags | u32 |
| energy | i64 |
| health, max_health | i32 |
| facing | u32 |
| entity_id | u64 |
| aux | u64 |

`kind`: `Empty=0`, `Solid=1`, `Creature=2`, `Corpse=3`, `Food=4`. New tile kinds = new `kind` values; old code sees unknown kinds as opaque (check `kind` before acting).

---

## Extensibility rules

1. **Never change envelope size** (64 B) or Header magic.
2. **Never re-order** existing Self/TileView fields — only append.
3. **New actions** = new `kind` in 128+ range first; promote to core when stable.
4. **New imports** = new names (`act2`), never change `act()` signature.
5. **Unknown `kind` in Action slot** → `act()` returns `-1`, no death (forward-compatible guest code).
6. **Spawn owner_id** must reference a live creature; child gets that creature's human account + `owner_id`.
7. **Deploy** → Init all zero; Self.owner_id = id.

---

## Gas & energy costs

Creatures pay **energy** (same unit as deploy credits and tile pools). Constants live in `crates/sim/src/abi.rs` and defaults in `SimConfig`. Economy rules (free-mint cap, food): [energy-budget.md](energy-budget.md).

### Units

| Symbol | Value | Meaning |
|--------|-------|---------|
| `ENERGY_SCALE` | 100 000 | 1 **glim** (display unit) |
| `CORPSE_ENERGY` | 10 × scale = **1 000 000** | Floor — at or below → death on next charge |
| `ACTION_ENERGY` | scale ÷ 4 = **25 000** | Default surcharge for spatial actions |

### Opcode gas (every tick)

Each think slice runs WASM until one of: successful `act()`, opcode budget exhausted, or `main` returns (suicide).

| Parameter | Default | Effect |
|-----------|---------|--------|
| `opcodes_per_tick` | 25 000 | Max WASM opcodes charged per tick |
| `energy_per_opcode` | 1 | Energy **destroyed** per opcode executed |

**Budget:** `min(opcodes_per_tick, energy ÷ energy_per_opcode)`. Host sets Wasmtime fuel to this budget; after the slice, `energy -= opcodes_used × energy_per_opcode` (ledger `record_destroy`).

| Outcome | Creature |
|---------|----------|
| Fuel exhausted mid-slice | **Suspend** — alive, no action this tick (`OutOfGas` path) |
| Energy ≤ corpse floor after charge | **Death** (`energy_floor`) |
| Trap / bad syscall args | **Death** (reason varies) |

Thinking costs scale with code size and loop depth; idle `sleep` loops pay almost nothing.

### Action surcharges (on successful `act()`)

Charged **in addition** to opcode gas. All go to `energy_destroyed`.

| Action (`kind`) | Extra energy | Notes |
|-----------------|--------------|-------|
| Move | `move_extra` (= 25 000) | Blocked move still pays surcharge if syscall accepted |
| Dig | `dig_extra` | |
| Place | `place_extra` | |
| Hit | `hit_extra` | + target damage via sim rules |
| Rotate | `rotate_extra` | |
| Eat | 0 | Transfers tile → creature |
| Spawn | 0 at syscall | Parent **transfers** `words[0]` energy to child |
| Signal / Broadcast | 0 | Payload only; range/target rules apply |

Tune via `PATCH /v1/dev/sim-config` (`move_extra`, …) in dev.

### Passive costs

| Mechanism | Default | When |
|-----------|---------|------|
| Health regen | `health_regen_cost` = 25 000 | Idle tick (no action, not full HP): +`health_regen` HP |
| Death leak | 20% of creature energy | `100 − CORPSE_YIELD_PERCENT`; remainder destroyed, yield → corpse tile |

### Spawn minimum

Child energy in spawn envelope must be **> `CORPSE_ENERGY`** (`SPAWN_MIN_ENERGY`). Parent debits full spawn energy on success.

### Signal / inbox

- Directed signal: target must exist and be within `r_sig` (default 5 hex).
- Inbox cap: `signal_inbox_cap` (default 8); oldest dropped.
- Signal/broadcast actions do not add an extra surcharge beyond opcode gas.

### Quick reference (default config)

```
1 tick, tight loop, no act():     ~ opcodes_used × 1 energy destroyed
1 tick, one move + 5k opcodes:    5 000 + 25 000 = 30 000 destroyed
Deploy floor:                     1 000 000 energy (10 glims corpse reserve)
```

## Creature ids

**u64** in ABI and wire. Accounts also receive **`account_creature_id`** (u64, not on map) for external control and signal routing — see [external-control.md](../external-control.md).

## External control (API key)

Owners may attach via **`GET /v1/control/ws`** with **`Authorization: Bearer tr_…`** only.

| Direction | Wire | ABI equivalent |
|-----------|------|----------------|
| Server → client | `{ "type": "recv", "sender", "envelope" }` | `recv()` → Inbox slot |
| Client → server | `{ "type": "signal", "target", "envelope" }` | Signal action |
| Client → server | `{ "type": "broadcast", "envelope" }` | Broadcast action |

Envelope is the same 64-byte shape documented above. Firebase JWT is **not** accepted on control WS.

## Why this shape

| Old idea | Problem | Now |
|----------|---------|-----|
| `action(tag, rel, a, b…g)` | 9 WASM params; awkward SDK | Guest writes **Action** slot; `act()` |
| `main(init_ptr)` | pointer ceremony | **Init** slot at fixed offset |
| `recv(ptr)` | pointer ceremony | **Inbox** slot |
| tag + rel + 48 B data | three layouts | one **envelope**, words interpreted by kind |
| scattered offsets (8192/8256) | hard to reason | one **ABI page** map |

---

## SDK surface (target)

```rust
// read memory
energy(), owner_id(), id(), tile(rel), init() -> Envelope

// write Action slot + syscall
act(envelope: Envelope) -> i32
recv() -> Option<(u64, Envelope)>
rand() -> u64
```

Helper builders: `Envelope::mv(rel)`, `Envelope::spawn(rel, energy, owner, child)`, etc.
