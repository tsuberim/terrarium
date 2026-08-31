# Energy budget

Terrarium's in-world economy is **deflationary**: energy is destroyed continuously by action costs and death leaks. **Free** energy (anything that mints into the sim without debiting credits or an existing pool) is capped so it can never outpace destruction.

**Rule:** for every **2** energy units destroyed in the sim, at most **1** free unit may be minted over the lifetime of the world.

Equivalently: `free_minted ≤ ⌊energy_destroyed × 0.5⌋`.

---

## Flow taxonomy

Every energy change is exactly one of:

| Class | Direction | Budget? | Examples |
|-------|-----------|---------|----------|
| **Import** | Outside → sim | No | Deploy (credits → creature), dev faucet → credits |
| **Transfer** | Pool → pool | No | Spawn (parent → child), eat corpse, suicide payout to creature |
| **Destroy** | Sim → void | — (feeds budget) | Opcode gas, action extras, health-regen cost, death leak |
| **Free mint** | Void → sim | **Yes** | Terrain food, future ambient pickups |

Imports are paid (or dev tooling) and intentionally increase total in-world energy. Transfers conserve total. Destruction decreases total and **credits the free-mint allowance**. Free mint increases total but only within the allowance.

### Net balance (excluding imports)

When no one deploys:

```
Δin_world = free_minted_this_period − destroyed_this_period
          ≤ 0.5 × destroyed_total − destroyed_this_period   (at cap)
          < 0   (strictly negative whenever destruction > 0)
```

So the world **always bleeds** unless players keep importing via deploy.

---

## What counts as destroyed

Monotonic counter `energy_destroyed`. Increment on every unit removed from the sim without a matching transfer recipient:

| Sink | When |
|------|------|
| Opcode gas | Each charged opcode in `charge_opcode_gas` |
| Move / dig / place / hit extras | On successful or attempted action (same as today) |
| Health regen | `health_regen_cost` when regen applies |
| Death leak | `creature_energy − corpse_yield` on death (20% at default `CORPSE_YIELD_PERCENT`) |
| Corpse despawn / admin clear | If we add timed corpse decay later |

**Not destroyed:** energy moving parent → child, corpse → eater, suicide → owner creature, or import from credits.

---

## What counts as free mint

Monotonic counter `energy_free_minted`. Increment when energy appears from nothing inside the sim:

| Source | Notes |
|--------|-------|
| **Food** (terrain) | Primary v1 free source; see below |
| Admin / cheat spawn | Dev only; still goes through gate in prod builds |
| Future: radiation, sun, etc. | Same gate |

**Not free mint:** deploy, spawn, eat (corpse or food — food mint happens at food **creation** or **regen**, not at eat).

---

## Ledger

Single authoritative struct, owned by the sim (kernel or server engine — kernel preferred so ticks stay pure):

```rust
pub struct EnergyLedger {
    /// Lifetime energy destroyed (monotonic).
    pub destroyed: i64,
    /// Lifetime free energy minted (monotonic).
    pub free_minted: i64,
}

impl EnergyLedger {
    pub const FREE_MINT_RATIO_NUM: i64 = 1;
    pub const FREE_MINT_RATIO_DEN: i64 = 2;

    pub fn free_budget(&self) -> i64 {
        (self.destroyed * Self::FREE_MINT_RATIO_NUM / Self::FREE_MINT_RATIO_DEN)
            .saturating_sub(self.free_minted)
    }

    /// Returns amount actually granted (may be less than requested).
    pub fn try_mint_free(&mut self, amount: i64) -> i64 {
        let grant = amount.max(0).min(self.free_budget());
        if grant > 0 {
            self.free_minted += grant;
        }
        grant
    }

    pub fn record_destroy(&mut self, amount: i64) {
        if amount > 0 {
            self.destroyed += amount;
        }
    }
}
```

Persist `destroyed` and `free_minted` in SQLite (`world_meta` or extend checkpoint) so restarts preserve the allowance curve.

---

## Per-tick accounting

Extend `TickResult`:

```rust
pub struct TickEnergyAccounting {
    pub destroyed: i64,
    pub free_minted: i64,
}
```

Kernel accumulates during `run_tick`:

1. Wrap energy mutations with deltas (destroy vs transfer).
2. On death: `destroyed += energy - corpse_yield_energy(energy)`.
3. On opcode/action charges: `destroyed += cost`.
4. Any free-mint path calls `ledger.try_mint_free(requested)` before crediting a creature or tile.

Server applies `TickEnergyAccounting` to the persisted ledger after each tick.

---

## Food (first free source)

Procedural terrain exposes **food** tiles — edible cells that grant energy on `eat`.

**Mint timing (recommended):** energy is reserved from the budget when food is **placed or refilled**, not when eaten. Eating only transfers food → creature (transfer class).

```
spawn food at (q,r) with nominal value V
  grant = ledger.try_mint_free(V)
  tile.energy = grant          // may be 0 if budget exhausted
```

Scavengers/predators compete for food; when budget is dry, new food appears empty until more action costs accumulate.

**Food lifecycle (sketch):**

1. Map generator scores cells (noise); top fraction become food **sites**.
2. On world init or chunk wake, sites try to fill from budget up to `FOOD_CAP` per cell.
3. Optional slow regen: each regen tick calls `try_mint_free` for a small drip.
4. `eat` on food: transfer `tile.energy` to creature, clear or reduce food.

---

## Gate failures

When `try_mint_free` returns less than requested:

| Context | Behavior |
|---------|----------|
| Food placement | Create food with `energy = grant` (partial or empty) |
| Food regen | Skip or partial fill |
| Admin spawn | Return error / partial to caller |

Never silently grant over budget. Partial grants are fine for terrain; API paths should surface errors.

---

## Observability

`EnergyLedger` is **internal** — persisted in SQLite for sim accounting and dev tooling, **not** exposed on WebSocket or REST. Operators can inspect via DB or a future admin panel.

Public wire (`full: true` delta, REST `/v1/world`) includes `deploy_cost`, `corpse_energy`, `sim_config`, creatures, and tiles only.

---

## Implementation phases

| Phase | Work |
|-------|------|
| **1 — Ledger** | `EnergyLedger` in kernel; `record_destroy` on existing sinks; persist columns; **internal only** (not on public wire) |
| **2 — Tick accounting** | `TickEnergyAccounting` in `TickResult`; tests for 2:1 cap |
| **3 — Food** | Simple periodic food spawn (no terrain noise yet); eat transfers; mint at fill |
| **4 — Balance pass** | Tune food density, nominal value, regen vs typical destroy rate |

Phase 1 can ship without food; the world simply accrues budget while free sources are empty.

---

## Constants (starting point)

| Constant | Value | Notes |
|----------|-------|-------|
| `FREE_MINT_RATIO` | 1:2 | Fixed; not a sim_config knob at first |
| `CORPSE_YIELD_PERCENT` | 80% | Already implemented; 20% → destroy |
| Node nominal value | TBD | e.g. `1 × ENERGY_SCALE` |
| Node site density | TBD | Fraction of passable cells |

---

## Open questions

- Should **deploy** count toward a separate import cap, or remain unlimited at fixed credit price?
- Corpse **decay** (destroy remainder on tile after N ticks) — adds destroy without transfer.
- Cross-world budget (if we ever shard): one global ledger vs per-region.
