# Dev-only strategy sources

Rust strategies compiled to WASM for **Predator** and **Scavenger** ecosystem examples in the sim seed. Not a player-facing deploy path — players use Creature Studio or upload WASM.

## Layout

- `hunter/` — shared hunt logic (scan vision, step toward, eat adjacent)
- `predator/` / `scavenger/` / `prey/` / `hawk/` — thin WASM exports (`main` lifetime loop)
- `tools/` — sync compiled WASM → WAT in sim examples

## Build

```bash
./scripts/build-strategies.sh
```

Requires `wasm32-unknown-unknown` (script runs `rustup target add` if needed).

Updates committed WAT in `crates/sim/src/examples.rs`. CI does **not** build strategies — run the script locally and commit after changing strategy logic.

## Pseudocode

```text
TARGET = creature (2) | corpse (3)

for (dx, dy) in adjacent:
    if sense_kind(dx, dy) == TARGET: eat(dir_of(dx, dy)); sleep(); return

for d in 1..=VISION:
    for (dx, dy) in ring(d):
        if sense_kind(dx, dy) == TARGET: step_toward(dx, dy); sleep(); return

sleep()
```
