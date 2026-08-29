# Terrarium WASM (WAT)

Creatures are **WebAssembly modules** written in WAT. The server compiles WAT to WASM on deploy. Code is immutable after deploy.

## Module contract

- Import namespace `"terrarium"` (see host syscalls below)
- Export `(func "tick")` — called once per sim tick
- Export `(memory "memory")` if using `recv` / `signal_to` (needs ≥ 36 bytes at ptr 0)

## Host syscalls

| Import | Signature | Effect |
|--------|-----------|--------|
| `sleep` | `() -> ()` | No-op, zero cost |
| `energy` | `() -> i64` | Current energy |
| `pos_x` / `pos_y` | `() -> i32` | Position |
| `sense_at` | `(i32 dx, i32 dy) -> i32` | Tile kind in vision square |
| `sense_energy` | `(i32 dx, i32 dy) -> i64` | Energy at cell |
| `move` | `(i32 dir) -> i32` | Queue move (dirs: N=0 E=1 S=2 W=3) |
| `dig` / `place` / `eat` | `(i32 dir) -> i32` | Queue action |
| `spawn` | `(i32 dir, i32 energy) -> i32` | Bud clone |
| `suicide` | `() -> ()` | Die, credit owner |
| `signal_broadcast` | `(i32 byte) -> i32` | Broadcast in R_sig |
| `signal_to` | `(i32 ptr, i32 byte) -> i32` | Directed signal (16-byte UUID at ptr) |
| `recv` | `(i32 ptr) -> i32` | 1 if message, else 0; writes 36-byte struct |
| `random_byte` | `() -> i32` | Pseudorandom byte 0–255 (seeded by creature id + sim tick) |
| `uptime` | `() -> i32` | Ticks alive since deploy/spawn |

## Tile kinds

`empty=0`, `solid=1`, `creature=2`, `corpse=3`

## Recv struct (little-endian, 36 bytes)

| Offset | Field |
|--------|-------|
| 0 | has_msg (always 1 when returned) |
| 4 | from_x |
| 8 | from_y |
| 12 | byte |
| 16 | broadcast (0 or 1) |
| 20 | from_id (16-byte UUID) |

Trap or out-of-energy → creature dies.

## Gas (EVM-style)

Each creature tick gets a **gas budget** of `opcodes_per_tick` (default 10_000). Every WASM instruction and every `call` to a host import consumes 1 opcode. Used opcodes cost `energy_per_opcode` energy (default **1** — cheap at million-scale units).

Gas budget is also capped by affordable energy: `floor(energy / energy_per_opcode)`. Out of gas traps → death.

Move/dig/place still cost separate action energy (`move_extra`, etc.) beyond gas.
