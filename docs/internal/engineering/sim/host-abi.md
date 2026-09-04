# Terrarium WASM (host ABI)

Creatures run as **WebAssembly modules**. Primary path: **Rust in Creature Studio** ([sdk/rust/README.md](../../../../sdk/rust/README.md)).

Code is immutable after deploy.

> Player docs: Mintlify [Host ABI](https://terrarium.mintlify.app/reference/host-abi). PRD: [../product/requirements.md](../product/requirements.md) §5–11.

## Module contract

- Import namespace `"terrarium"` (see host syscalls below)
- Export `(func "main")` — host **starts `main` once** per creature life and **resumes** the same WASM fiber each sim tick (heap + call stack preserved)
- Export `(memory "memory")` if using `recv`, `signal_to`, or `sense` (needs scratch bytes at ptr)

**Slice execution.** Each sim tick the host refills opcode gas and resumes WASM until the slice ends:

| Slice end | World effect |
|-----------|--------------|
| First successful world action (`move`, `rotate`, `eat`, …) | One action applied; execution **suspends** (async yield) |
| Opcode gas exhausted, no action taken | **Suspend** — creature stays alive; resumes next tick |
| `main` returns or `break` out of the program loop | **Suicide** — creature dies; energy payout to owner |
| Real WASM trap (OOB, bad direction, …) | Death |

Studio source is a **lifetime program**: write statements (`move_forward();`), a `loop { ... }`, or `pub fn main() { ... }`. The compile worker injects `terrarium_sdk::prelude::*` and wraps the body in `pub fn main()` when needed. Bare statements run once — after the slice resumes and `main` returns, the creature halts (suicide). At most one world action per tick (second calls no-op). Sensing, `sleep`, and reads are unlimited within the gas budget.

## Host syscalls

| Import | Signature | Effect |
|--------|-----------|--------|
| `sleep` | `() -> ()` | No-op, zero cost |
| `energy` | `() -> i64` | Current energy |
| `health` | `() -> i64` | Current health |
| `pos_x` / `pos_y` | `() -> i32` | Axial q/r position |
| `facing` | `() -> i32` | Body facing 0–5 (E, NE, NW, W, SW, SE) |
| `rotate` | `(i32 delta) -> i32` | Turn by `delta` hex steps (clockwise = +1); facing updates end of tick |
| `sense` | `(i32 dq, i32 dr, i32 ptr) -> i32` | Write cell snapshot at `ptr`; returns 1 if in FOV, else 0 |
| `move` | `(i32 rel) -> i32` | Step **forward** onto an empty cell only (`rel=0`); blocked by solid, food, corpse, or creatures |
| `dig` / `place` | `(i32 rel) -> i32` | Act on **forward** adjacent cell only (`rel=0`) |
| `eat` | `(i32 rel) -> i32` | Eat corpse or food on **forward** cell only (`rel=0`) |
| `hit` | `(i32 rel) -> i32` | Hit creature on **forward** cell only (`rel=0`; costs energy) |
| `spawn` | `(i32 rel, i32 energy) -> i32` | Bud clone on **forward** empty cell only (`rel=0`) |
| `signal_broadcast` | `(i32 byte) -> i32` | Broadcast in R_sig |
| `signal_to` | `(i32 ptr, i32 byte) -> i32` | Directed signal (16-byte UUID at ptr) |
| `recv` | `(i32 ptr) -> i32` | 1 if message, else 0; writes 36-byte struct |
| `random_byte` | `() -> i32` | Pseudorandom byte 0–255 (seeded by creature id + sim tick) |
| `uptime` | `() -> i32` | Ticks alive since deploy/spawn |

## Tile kinds (`sense` struct field `kind`)

`empty=0`, `solid=1`, `creature=2`, `corpse=3`, `food=4`

## Sense struct (little-endian, 24 bytes)

| Offset | Field |
|--------|-------|
| 0 | kind (i32) |
| 4 | orientation (i32) — creature facing 0–5 when kind=creature, else −1 |
| 8 | energy (i64) — creature, corpse, or food energy in raw units (÷ 100 000 = **glims**, ◆) |
| 16 | health (i32) — live creature only |
| 20 | max_health (i32) — live creature only |

## Recv struct (little-endian, 36 bytes)

| Offset | Field |
|--------|-------|
| 0 | has_msg (always 1 when returned) |
| 4 | from_q (pos_x of sender) |
| 8 | from_r (pos_y of sender) |
| 12 | byte |
| 16 | broadcast (0 or 1) |
| 20 | from_id (16-byte UUID) |

Host import traps for out-of-energy, energy floor, bad direction, etc. → creature dies. Opcode **fuel exhaustion** during a slice → **suspend** (next sim tick resumes), not death.

## Gas (EVM-style)

Each sim tick gets a **gas budget** of `opcodes_per_tick` (default 10_000). Every WASM instruction and every `call` to a host import consumes 1 opcode. Used opcodes cost `energy_per_opcode` energy (default **1** — cheap at million-scale units).

Gas budget is also capped by affordable energy: `floor(energy / energy_per_opcode)`. Running out of fuel mid-slice suspends; zero affordable budget at tick start still kills (`out_of_gas`).

Move/dig/place still cost separate action energy (`move_extra`, etc.) beyond gas.
