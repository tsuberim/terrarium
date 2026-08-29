# Architecture

Terrarium is a single authoritative world server with many read-only spectators. The design separates **simulation**, **fan-out**, and **persistence** so each can scale independently.

## Core invariant

> **The world clock advances at exactly `TICK_HZ` (10 Hz) for every client**, regardless of how many spectators are connected or how heavy a single tick's computation is.

Clients never drive the sim. The sim never waits on clients or SQLite.

## Process model

```
┌──────────────────────────────────────────────────────────────┐
│  tokio runtime (HTTP, WebSocket, auth, deploy, checkpoint)   │
│                                                              │
│  ┌─────────────┐   broadcast    ┌─────────────────────────┐  │
│  │ broadcast   │───────────────►│ WS task per client (N)  │  │
│  │ channel     │   (non-block)  │ JSON encode + send      │  │
│  └──────▲──────┘                └─────────────────────────┘  │
│         │                                                    │
│  ┌──────┴──────┐   mpsc queue   ┌─────────────────────────┐  │
│  │ checkpoint  │◄───────────────│ persist worker (async)  │  │
│  │ worker      │                └─────────────────────────┘  │
└─────────┼────────────────────────────────────────────────────┘
          │
┌─────────▼────────────────────────────────────────────────────┐
│  `terrarium-sim` thread (dedicated, fixed timestep)          │
│  loop: tick_step → broadcast delta → sleep until next slot   │
│  WorldEngine in RAM (parking_lot RwLock)                     │
└──────────────────────────────────────────────────────────────┘
          │
┌─────────▼────────────────────────────────────────────────────┐
│  terrarium-kernel — pure CPU, no I/O                         │
└──────────────────────────────────────────────────────────────┘
```

### Why three lanes?

| Lane | Must not block |
|------|----------------|
| **Sim thread** | WebSocket sends, SQLite, HTTP |
| **Broadcast** | Slow clients (lagging subs get snapshot resync) |
| **Checkpoint** | Sim thread (queued, ~1 Hz) |

## Fixed timestep

The sim thread uses wall-clock scheduling:

1. Record `start = Instant::now()`
2. Run `tick_step()` (kernel only)
3. Push delta to broadcast (**every tick**, even if empty — clients use `tick` as clock)
4. Queue checkpoint snapshot if due
5. `sleep(max(0, period - elapsed))`
6. If `elapsed > period` → log **tick overrun** (sim complexity exceeded budget)

**Important:** we do not burst-catch-up missed ticks. If the sim falls behind, the world clock slows slightly (sleep goes to 0) but never runs 2× ticks in one slot. Long-term fix for overruns is **sim budgets** (below), not faster wall clock.

## Fan-out to many clients

- `tokio::sync::broadcast` with capacity **4096** — send is O(1), copies Arc-like clone per subscriber
- Each WebSocket runs in its own async task; JSON encode happens **per client**, not on sim thread
- Slow client → `RecvError::Lagged` → full snapshot resync (already implemented)
- Sim thread never awaits network

### Scale limits today

| Clients | Bottleneck | Mitigation (future) |
|---------|------------|---------------------|
| ~100 | JSON encode per client | Binary codec (MessagePack/CBOR) |
| ~1k | Bandwidth (full deltas) | Viewport subscription (interest mgmt) |
| ~10k+ | CPU on encode | Dedicated relay / CDN edge; chunk deltas |

## Wire protocol

Connect: `GET /api/v1/world/ws`

**Snapshot** (connect + lag recovery):

```json
{ "type": "snapshot", "tick": 1204, "deploy_cost": 100, "creatures": [...], "tiles": [...] }
```

**Delta** (every sim tick — `tick` always present):

```json
{
  "type": "delta",
  "tick": 1205,
  "creatures_upsert": [],
  "creatures_remove": [],
  "tiles_upsert": [],
  "tiles_remove": []
}
```

Empty arrays = heartbeat; client still advances interpolation clock.

## When sim complexity grows

Today all creatures tick every frame. Path to **tons of entities** without breaking Hz:

### Phase A — Dirty tracking (next)
- Don't clone tile map each tick; track changed cells in kernel
- Spatial hash for creature collision (O(1) not O(n²))

### Phase B — Time budgets
- `TICK_BUDGET_US` (default 100ms @ 10Hz)
- If overrun: process remaining creatures next tick (fair round-robin cursor)
- Idle/sleeping creatures cost ~0 — skip VM when at `sleep`/`halt`

### Phase C — Chunked world
- 32×32 cell chunks; only tick chunks with awake creatures
- Clients subscribe to `{chunk_x, chunk_y}` set based on camera viewport
- Server sends chunk-scoped deltas → bandwidth ∝ viewport, not world size

### Phase D — Horizontal (far future)
- Single writer per world (this design); read replicas for spectating regions
- Not sharding the same world — one authoritative process

## Persistence

- **Deploy / auth / credits** → SQLite immediately + engine insert
- **Sim state** → checkpoint every `PERSIST_EVERY_TICKS` (default 10 = 1s)
- **Restart** → bootstrap engine from SQLite

## Config

| Env | Default | Meaning |
|-----|---------|---------|
| `PERSIST_EVERY_TICKS` | `10` | Checkpoint interval |
| `TICK_HZ` | `10` (kernel const) | World clock rate |
| `DATABASE_URL` | file sqlite | Persistence |

## Files

| Path | Role |
|------|------|
| `crates/server/src/engine.rs` | RAM world, sim thread, broadcast |
| `crates/server/src/ws.rs` | Per-client fan-out |
| `crates/kernel/` | VM, energy ledger (planned), no I/O |
| `docs/energy-budget.md` | Free-mint budget (2:1 destroy ratio) |
| `apps/skin/src/hooks/useWorldStream.ts` | Client delta apply + maps |
