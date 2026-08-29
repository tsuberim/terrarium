# Terrarium — Vision

Terrarium is a 2D top-down MMO and a platform for deploying programmable creatures into a persistent, always-running real-time world. It is a simulation first — not a game with scores, factions, or win conditions. People assign their own meaning.

Energy is the sole resource. It enters the system when people pay money for credits. Credits and energy are the same thing at a fixed exchange rate. Everything in-world is driven by a minimal set of primitives; complex structures and strategies should emerge naturally from simple rules and creature programs.

---

## Principles

- **Bare bones.** Resist adding primitives until multiple emergent behaviors cannot happen without them.
- **Conserved energy.** Energy is only destroyed by action costs. All other flows redistribute it.
- **Real stakes.** The world is persistent. Death matters. Code is immutable after deploy.
- **Emergence over mechanics.** No territories, factions, leaderboards, or combat stats. Basic physics and primitives only.
- **Open spectating.** Everyone can watch the world in real time. Source code is private to the owner.

---

## World

- **Topology:** Huge bounded 2D grid with torus wrapping (no hard edges).
- **Persistence:** Always on. Full world state persists across restarts.
- **Authority:** Single global world server (no shards for now).
- **Representation:** Tile grid is the spatial model. Creatures may render at sub-cell positions, but block operations snap to cells.

### Cell types (v1)

| Type   | Description |
|--------|-------------|
| `empty` | Passable. Default. |
| `solid` | Impassable. Placed and removed by creatures. |

No additional block types, overlays, or terrain features at launch.

### Entities

| Type      | Description |
|-----------|-------------|
| Creature  | Programmable agent with energy, owner, and immutable code. |
| Corpse    | Remains when a creature dies. Holds remaining energy. Must be explicitly eaten. |

---

## Creatures

### Programming

- Code is **immutable** after deploy or spawn.
- Code is **private** — only the owner can read it.
- Creatures choose when to **think** (run code); they can **sleep** indefinitely at zero cost.
- Sleep may be interruptible (mechanism TBD).

### Actions (v1 surface)

| Action       | Notes |
|--------------|-------|
| Move         | Costs energy. |
| Place        | Set adjacent cell to `solid`. Costs energy. |
| Dig          | Set adjacent cell to `empty`. Costs energy. |
| Eat          | Consume corpse or other energy source on a tile. |
| Spawn        | Parent submits new code; transfers energy to child (≥ spawn minimum). |
| Signal       | Communicate with nearby creatures (exact mechanism TBD). |
| Sleep / Wake | Sleep costs nothing. Wake to think/act. |
| Suicide      | Transfer all carried energy to owner. |

Relative action costs and the minimal sense model are TBD. The action set itself may shrink or grow only when a primitive is provably necessary.

### Sensing (v1)

- Contents of nearby / adjacent cells (terrain, creatures, corpses).
- Incoming signals from other creatures.

---

## Energy & Economy

| Rule | Detail |
|------|--------|
| Exchange rate | Fixed credits ↔ energy conversion. |
| Entry | Energy enters only via paid credits (no ambient generation). |
| Action cost | Every action spends energy (movement, thinking, sensing, etc.). |
| Spawn minimum | Minimum energy to deploy or spawn; also the death threshold. |
| Death | Energy below threshold → creature dies → corpse with remaining energy. |
| Eat | Explicit action required to consume a corpse. |
| Suicide | All carried energy goes to owner. |
| Cash out | When owner is a human account, suicide converts energy to credits. No fees or minimums at first. |

### Deploy vs spawn

| Path | Paid by | Code source |
|------|---------|-------------|
| **Deploy** (human) | Account credits at spawn location of owner's choice | Human submits code |
| **Spawn** (creature) | Parent's in-world energy (transferred to child) | Parent creature submits code |

---

## Ownership

- A human **account** can deploy as many creatures as it can afford.
- When creature A spawns B, **A owns B** (A sees B's code).
- Unlimited lineage: B can spawn C, and so on.
- When a creature dies, **ownership of its children walks up the chain** to the parent, then the grandparent, up to the top-level human account if needed.
- Suicide payout to a **creature** owner is received as **in-world energy** (spendable on spawn and actions).
- Suicide payout to a **human** owner is received as **credits**.

---

## Clients

- **Spectator:** Real-time view for everyone. God view or follow-a-creature camera.
- **Deploy / manage:** API and game UI (details TBD).
- No public read access to creature source code.

---

## Explicit non-goals (v1)

- Shards or multi-region worlds
- Factions, territories, or opt-in PvP rules
- Scoreboards or leaderboards
- Block types beyond `empty` / `solid`
- Ambient energy generation
- Public code inspection

---

## Open questions

- Exact signal / communication primitive
- Relative energy costs per action (move, think, sense, place, dig, spawn)
- Sleep interrupt conditions
- Payment provider and cash-out mechanics
- MVP scope (after initial devops)
