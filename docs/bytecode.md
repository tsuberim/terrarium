# Terrarium WASM (WAT)

Creatures are **WebAssembly modules** written in WAT. The server compiles WAT to WASM on deploy. Code is immutable after deploy.

## Module contract

- Import namespace `"terrarium"` (see host syscalls below)
- Export `(func "tick")` — called once per sim tick
- Export `(memory "memory")` if using `recv`, `signal_to`, or `sense` (needs scratch bytes at ptr)

## Host syscalls

| Import | Signature | Effect |
|--------|-----------|--------|
| `sleep` | `() -> ()` | No-op, zero cost |
| `energy` | `() -> i64` | Current energy |
| `health` | `() -> i64` | Current health |
| `pos_x` / `pos_y` | `() -> i32` | Axial q/r position |
| `sense` | `(i32 dq, i32 dr, i32 ptr) -> i32` | Write cell snapshot at `ptr`; returns 1 |
| `move` | `(i32 dir) -> i32` | Queue move (dirs: E=0 NE=1 NW=2 W=3 SW=4 SE=5) |
| `dig` / `place` | `(i32 dir) -> i32` | Queue action |
| `eat` | `(i32 dir) -> i32` | Consume adjacent corpse (or future edible tile) |
| `hit` | `(i32 dir) -> i32` | Damage adjacent live creature (costs energy) |
| `spawn` | `(i32 dir, i32 energy) -> i32` | Bud clone |
| `suicide` | `() -> ()` | Die, credit owner |
| `signal_broadcast` | `(i32 byte) -> i32` | Broadcast in R_sig |
| `signal_to` | `(i32 ptr, i32 byte) -> i32` | Directed signal (16-byte UUID at ptr) |
| `recv` | `(i32 ptr) -> i32` | 1 if message, else 0; writes 36-byte struct |
| `random_byte` | `() -> i32` | Pseudorandom byte 0–255 (seeded by creature id + sim tick) |
| `uptime` | `() -> i32` | Ticks alive since deploy/spawn |

## Tile kinds (`sense` struct field `kind`)

`empty=0`, `solid=1`, `creature=2`, `corpse=3`

## Sense struct (little-endian, 24 bytes)

| Offset | Field |
|--------|-------|
| 0 | kind (i32) |
| 8 | energy (i64) — creature or corpse energy; 0 on empty/solid |
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

Trap or out-of-energy → creature dies.

## Gas (EVM-style)

Each creature tick gets a **gas budget** of `opcodes_per_tick` (default 10_000). Every WASM instruction and every `call` to a host import consumes 1 opcode. Used opcodes cost `energy_per_opcode` energy (default **1** — cheap at million-scale units).

Gas budget is also capped by affordable energy: `floor(energy / energy_per_opcode)`. Out of gas traps → death.

Move/dig/place still cost separate action energy (`move_extra`, etc.) beyond gas.
