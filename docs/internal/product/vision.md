# Terrarium — Vision

> **Shipped behavior:** [requirements.md](requirements.md) (PRD).  
> **Economy:** [../engineering/sim/energy-budget.md](../engineering/sim/energy-budget.md). **ABI:** [../engineering/sim/host-abi.md](../engineering/sim/host-abi.md).

Terrarium is a **persistent 2D hex-world simulation** where players deploy **programmable creatures** (WASM) that compete for energy. It is a simulation first — not a game with scores, factions, or win conditions. People assign their own meaning.

Energy is the sole resource. It enters when players pay for credits (1:1 with in-world energy on deploy). Complex behavior should **emerge** from simple rules and creature code — not from bespoke game mechanics.

---

## Principles

- **Bare bones** — no new primitives until multiple behaviors require them
- **Deflationary sim** — action costs and death destroy energy; free mint capped at 1:2 destroy ratio ([../engineering/sim/energy-budget.md](../engineering/sim/energy-budget.md))
- **Real stakes** — persistent world; death matters; code immutable after deploy
- **Emergence over mechanics** — no territories, factions, leaderboards, combat stats
- **Open spectating, private code** — everyone watches live; source stays with the owner

---

## World (summary)

| Aspect | Direction |
|--------|-----------|
| Topology | Sparse hex grid (axial q/r); no wrap-around |
| Authority | Single global world @ 2 Hz |
| Persistence | File SQLite locally; prod ephemeral today ([../engineering/tech-debt.md](../engineering/tech-debt.md) TD-INF-1) |
| Cells | empty, solid, corpse, food — creatures via `sense` only |
| Client | WS deltas + `actions`/`events` for motion FX ([../engineering/architecture.md](../engineering/architecture.md)) |

Cell types, sensing, actions, deploy/spawn, economy tables → **PRD §4–6**.

Player how-to: [public Mintlify](https://terrarium.mintlify.app/getting-started/studio).

---

## Non-goals (v1)

Do not build without explicit product decision — full list in PRD §15:

- Shards / multi-region
- Factions, territories, PvP rulesets
- Leaderboards
- Torus map
- Public source inspection
- In-place code updates
- Staging environment

---

## Open questions

Undecided sim/product rules — **do not implement until chosen**. Canonical list: PRD §16.

Examples: sleep interrupt conditions, payment/cash-out, corpse decay, prod persistent DB, compile-worker always-on.

When a question is resolved, update the PRD and remove or close the matching tech-debt item.
