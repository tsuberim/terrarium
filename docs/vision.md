# Terrarium — Vision

Terrarium is a 2D top-down MMO and a platform for deploying programmable creatures into a persistent, always-running real-time world. It is a simulation first — not a game with scores, factions, or win conditions. People assign their own meaning.

Energy is the sole resource. It enters the system when people pay money for credits. Credits and energy are the same thing at a fixed exchange rate. Everything in-world is driven by a minimal set of primitives; complex structures and strategies should emerge naturally from simple rules and creature programs.

---

## Principles

- **Bare bones.** Resist adding primitives until multiple emergent behaviors cannot happen without them.
- **Deflationary sim.** Action costs and death leaks destroy energy. Free in-world sources (food, etc.) are budgeted: at most **1 unit minted per 2 units destroyed** — see [energy-budget.md](energy-budget.md).
- **Real stakes.** The world is persistent. Death matters. Code is immutable after deploy.
- **Emergence over mechanics.** No territories, factions, leaderboards, or combat stats. Basic physics and primitives only.
- **Open spectating.** Everyone can watch the world in real time. Source code is private to the owner.

---

## World

- **Topology:** Sparse hex grid (axial q/r). No torus wrapping today — open coordinates, sparse tile map.
- **Persistence:** Always on. Full world state persists across restarts when using a file-backed SQLite DB. Production Cloud Run currently uses ephemeral in-memory SQLite (resets on redeploy) — acceptable for now.
- **Authority:** Single global world server (no shards for now).
- **Representation:** Creatures occupy one hex cell; client interpolates motion between ticks.

### Cell types (implemented)

| Kind | `sense` value | Description |
|------|---------------|-------------|
| `empty` | 0 | Passable default |
| `solid` | 1 | Impassable; placed/dug by creatures |
| `creature` | 2 | Live creature (via sense only) |
| `corpse` | 3 | Death remains; must be eaten for energy |
| `food` | 4 | Budgeted free energy; eaten like corpses |

### Entities

| Type | Description |
|------|-------------|
| Creature | Programmable agent with energy, health, owner, **facing**, immutable WASM code |
| Corpse | Tile left on death; holds ~80% of energy; explicit `eat` required |
| Food | Procedural edible tiles; mint gated by energy ledger |

---

## Creatures

### Programming

- Code is **immutable** after deploy or spawn.
- Code is **private** — only the owner can read it.
- **One action per tick** — at most one of move, rotate, eat, hit, dig, place, spawn, signal, suicide.
- Creatures choose when to **think** (run WASM); `sleep` is free.

See [bytecode.md](bytecode.md) for the host ABI.

### Body & orientation

- Each creature has **facing** 0–5 (E, NE, NW, W, SW, SE).
- **`rotate(delta)`** turns in place (costs energy); applied end of tick.
- **Forward actions** — `move`, `eat`, `hit`, `dig`, `place`, `spawn` act on the **forward** adjacent cell only (`rel=0`). Use `rotate` to aim first.

### Actions (v1 surface)

| Action | Notes |
|--------|-------|
| Move | Step forward onto empty cell only |
| Rotate | Turn body; no translation |
| Eat | Consume forward corpse or food |
| Hit | Damage forward live creature |
| Dig / Place | Modify forward cell |
| Spawn | Bud clone on forward empty cell |
| Signal | Broadcast or directed byte in `r_sig` |
| Sleep / Suicide | Sleep free; suicide credits owner |

### Sensing (implemented)

- **`sense(dq, dr)`** — cell contents within hex range **`r_vis`** (default 5) **and** frontal cone **`vis_half_arc`** (default 1 → ±60°, 120° total arc centered on facing).
- Out of range or outside cone → returns 0 (no trap).
- Sense struct includes kind, facing (for creatures), energy, health.
- Incoming signals via `recv`.

---

## Signal ecosystem (showcase)

| Byte | Who | Meaning |
|------|-----|---------|
| `0x01` | Prey | Alarm when fleeing — attracts **Hawk** and **Scavenger** |
| `0x02` | Predator | Hunt ping while chasing prey in vision |

Deploy a mix to see competition: prey alarms pull hawks and scavengers toward the fight; predators mark active hunts.

---

## Energy & Economy

| Rule | Detail |
|------|--------|
| Exchange rate | Fixed credits ↔ energy conversion |
| Entry | Paid deploy imports energy from credits **1:1** (`corpse_energy` floor + extra) |
| Free sources | Food mint gated 2:1 — [energy-budget.md](energy-budget.md) |
| Action cost | Gas + per-action extras (`move_extra`, `rotate_extra`, …) |
| Spawn minimum | `corpse_energy` floor; at/below → death |
| Death | ~80% to corpse tile; ~20% destroyed |
| Eat | Explicit; transfers tile energy to eater |
| Suicide | Energy to owner (credits if human, in-world if creature parent) |

### Deploy vs spawn

| Path | Paid by | Code source |
|------|---------|-------------|
| **Deploy** (human) | Account credits (`corpse_energy + extra`, 1:1 import) | Human submits WAT/WASM |
| **Spawn** (creature) | Parent's energy | Parent's code copied to child |

---

## Clients

- **Spectator:** Real-time view. God view or follow-a-creature camera. Animation driven by WS `actions` + `events` — see [architecture.md](architecture.md).
- **Deploy / manage:** API + game UI.
- No public read access to creature source code or WASM fingerprints on the wire.

Wire: delta-only WebSocket; `full: true` on connect/resync.

---

## Explicit non-goals (v1)

- Shards or multi-region worlds
- Factions, territories, or opt-in PvP rules
- Scoreboards or leaderboards
- Torus / wrap-around edges
- Public code inspection

---

## Open questions (future design — not bugs)

These are intentionally undecided product/sim rules. Implement only when we choose a direction:

- Sleep interrupt conditions
- Payment provider and cash-out mechanics
- Corpse decay (destroy remainder on tile after N ticks)
- Relative energy cost tuning at scale
