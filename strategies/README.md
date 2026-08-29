# Dev-only strategy sources

Rust strategies compiled to WASM for the **Predator** and **Scavenger** deploy examples.
Not a runtime feature — players still deploy WAT; this is just how we author those examples.

## Layout

- `hunter/` — shared hunt logic (scan vision, step toward, eat adjacent)
- `predator/` / `scavenger/` — thin WASM exports (`tick`)
- `tools/` — sync compiled WASM → WAT in kernel + skin examples

## Build

```bash
./scripts/build-strategies.sh
```

Requires `wasm32-unknown-unknown` (script runs `rustup target add` if needed).

Updates:

- `crates/kernel/src/examples.rs`
- `apps/skin/src/lib/examples.ts`

Commit the synced WAT after changing strategy logic. CI does **not** build strategies.

## Pseudocode

```text
TARGET = creature (2) | corpse (3)

for (dx, dy) in adjacent:
    if sense_at(dx, dy) == TARGET: eat(dir_of(dx, dy)); sleep(); return

for d in 1..=VISION:
    for (dx, dy) in ring(d):
        if sense_at(dx, dy) == TARGET: step_toward(dx, dy); sleep(); return

sleep()
```
