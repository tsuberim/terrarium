# External creature control (API key)

Lets account holders attach **external compute** to creatures they own: read inbox, send signals, using the **same 64-byte envelope** as the in-world ABI (`recv` / Signal / Broadcast).

> PRD: [../product/requirements.md](../product/requirements.md) §13. Envelope: [sim/host-abi.md](sim/host-abi.md).

## Auth

**API key only** on the control WebSocket. No Firebase session cookie, no anonymous access.

```
Authorization: Bearer tr_<secret>
```

- Keys minted in Studio (`POST /v1/api-keys`) while signed in with Firebase.
- Control WS **rejects** Firebase JWT — forces programmatic, revocable access.
- Key resolves to `owner_uid`; server verifies `creature.owner_uid == owner_uid` for the attached creature.

REST `/v1/me` may expose `account_creature_id` with Firebase or API key; control attach requires API key.

## Account creature id

Every account gets a stable **`account_creature_id: u64`** (minted once, stored on `accounts`).

| Role | Notes |
|------|-------|
| Identity in signal graph | External clients signal *as* the account id or *on behalf of* a deployed creature |
| Not on the map | No sim body unless a future feature adds one |
| Unique | Same u64 namespace as live creatures; never collides (server mints) |

Deployed creatures keep their own `id`. `Self.owner_id` on a creature may point at account id or another creature (spawn).

## Control WebSocket

**Separate from** spectator `GET /v1/world/ws` (read-only deltas, optional auth).

```
GET /api/v1/control/ws?creature_id=<u64>
Authorization: Bearer tr_...
```

One session **per connection** attached to one **live, owned** creature. Multiple connections to the same creature allowed (fan-out inbox; send serialized per creature).

### Session open

Server verifies:

1. API key valid → `owner_uid`
2. Creature exists, alive, `creature.owner_uid == owner_uid`
3. Upgrades to WebSocket

First message **server → client**:

```json
{
  "type": "attached",
  "creature_id": 9001,
  "account_creature_id": 42,
  "tick": 1205
}
```

Optional periodic **`state`** messages mirror ABI **Self** + inbox_len (same fields as host-abi Self region).

### Envelope on the wire

JSON encoding of the fixed **64-byte envelope** (host-abi):

```json
{
  "kind": 8,
  "pad": 0,
  "words": ["0", "0", "0", "0", "0", "0", "0"]
}
```

`words` are seven u64 strings (decimal or `0x` hex). Host and client convert to little-endian bytes.

## Client → server

| `type` | Body | Effect |
|--------|------|--------|
| `signal` | `{ "target": u64, "envelope": Envelope }` | Queue directed signal **as the attached creature** (same rules as in-sim Signal: range, target alive, one per tick if routed through action queue) |
| `broadcast` | `{ "envelope": Envelope }` | Broadcast **as attached creature** within `r_sig` |
| `ping` | `{}` | `pong` |

Invalid envelope, out-of-range target, or unknown creature → error frame; **does not** kill the creature (external path is best-effort).

### Rate limits (initial)

- Max **32** outbound control messages per creature per sim tick
- Max **4** concurrent control WS per creature
- Oversize JSON → close with `4400`

## Server → client

| `type` | When |
|--------|------|
| `recv` | Inbox delivery to attached creature: `{ "sender": u64, "envelope": Envelope }` |
| `state` | Optional tick snapshot (Self fields) |
| `detached` | Creature died or ownership lost; close |
| `error` | `{ "code", "message" }` |

**`recv` mirrors ABI `recv()`**: same sender u64 + 64-byte envelope the WASM module would read from the Inbox slot after `recv()`.

Spectator WS **does not** expose raw inbox (privacy); control WS does for owners only.

## Flow

```
External bot                    Server                         Sim
    |                              |                              |
    |-- WS + API key attach ------>| verify owner                 |
    |<-- attached -----------------|                              |
    |                              |<---- tick: creature inbox ---|
    |<-- recv {sender, env} ------| fan-out from sim Signal      |
    |-- signal {target, env} ----->| inject as creature action -->|
    |                              |                              |
```

Injection path: control messages enqueue on the creature's **next tick** think slice (same slot as guest-written Action envelope with kind Signal/Broadcast). External logic can run off-device; WASM may still run in parallel unless a future **remote brain** mode disables it.

## Security

- API key only; keys revocable (`DELETE /v1/api-keys/{id}`)
- Attach only to **own** creatures (`owner_uid` match)
- No read of other players' inbox or source
- Envelope size capped at 64 bytes; no binary blobs on control WS
- Log `last_used_at` on key (existing) + optional per-attach audit

## Implementation status

**shipped** — `GET /api/v1/control/ws?creature_id=<u64>` with `Authorization: Bearer tr_…` only.

| Area | Status |
|------|--------|
| Spectator WS | Unchanged — no auth; inbound ignored |
| Control WS | API key attach; `recv` / `signal` / `broadcast` |
| `GET /v1/me` | Includes `account_creature_id` |
| Creature ids | u64 on wire (JSON string), DB INTEGER |
| Deploy | Sets `owner_id = id`, zeros Init |

## Open questions

| Topic | Default lean |
|-------|----------------|
| External `act()` (move/eat/…) | Phase 2; v1 is signal/recv only |
| Replace WASM with remote brain | Phase 3 |
| `state` push every tick vs on change | On change + tick boundary |
