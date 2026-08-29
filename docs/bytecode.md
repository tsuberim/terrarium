# Terrarium bytecode (v0)

Creatures run a tiny stack machine. You write **assembly** (text); the server assembles it to bytecode on deploy. Code is immutable after deploy.

## Model

- **Stack:** 256 slots, 32-bit signed values. Overflow/underflow halts the creature for that tick.
- **Directions:** `n` `e` `s` `w` (north = −y, east = +x). The world wraps (torus).
- **Ticks:** The kernel runs at 10 Hz. Each executed instruction costs **1 energy** unless noted.
- **Actions** (`move`, `dig`, `place`, `eat`) cost **extra energy** when the kernel applies them (TBD).

## Tile kinds (`sense` pushes)

| Value | Meaning |
|------:|---------|
| 0 | empty |
| 1 | solid |
| 2 | creature |
| 3 | corpse |

## Instructions

| Mnemonic | Bytes | Stack | Effect |
|----------|------:|-------|--------|
| `halt` | 1 | — | Stop until woken |
| `sleep` | 1 | — | Yield tick (**0** energy) |
| `move d` | 2 | — | Move one cell (`d` = n/e/s/w) |
| `dig d` | 2 | — | Clear adjacent cell to empty |
| `place d` | 2 | — | Set adjacent cell to solid |
| `eat d` | 2 | — | Eat corpse / energy on adjacent cell |
| `sense d` | 2 | → kind | Push tile kind adjacent to `d` |
| `energy` | 1 | → n | Push this creature's energy |
| `pop` | 1 | −1 | Drop stack top |
| `dup` | 1 | +1 | Copy stack top |
| `push n` | 3 | → n | Push 16-bit immediate |
| `jmp label` | 3 | — | Relative jump |
| `jz label` | 3 | −1 | Jump if top == 0 |
| `jnz label` | 3 | −1 | Jump if top != 0 |
| `eq` | 1 | −1 → 0/1 | `(a == b)` |
| `lt` | 1 | −1 → 0/1 | `(a < b)` |
| `add` | 1 | −1 → sum | |
| `sub` | 1 | −1 → diff | `b - a` (pop `a` then `b`) |
| `suicide` | 1 | — | Die; energy to owner |

Labels: `name:` at line start. Comments: `; …`

## Example — idle

```asm
loop:
  sleep
  jmp loop
```

## Example — tunnel east

```asm
start:
  move e
  dig e
  sleep
  jmp start
```

## Example — wall if blocked north

```asm
  sense n
  push 1        ; solid
  eq
  jz place_it
  jmp done
place_it:
  place n
done:
  sleep
  jmp done
```

`jz place_it` runs when north is **not** solid.

Programs run at 10 Hz after deploy. Each tick executes until `sleep` or `halt`.
