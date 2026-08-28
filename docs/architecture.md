# Architecture

Two pieces, on purpose.

**Kernel** runs the world. **Skin** is a camera. Later, the same kernel is the multiplayer server. Nobody plays through a menu.

## Kernel

Rust. Deterministic fixed-point 2D physics. Mass accounting is load-bearing.

Creatures run a tiny bytecode program inside the kernel (WASM guest modules can come later). Mass is their fuel. Guest code does not get free compute: cycles spent are mass spent, burned to the house.

v1 of the kernel compiles to WASM and also runs natively. The web skin hosts the same crate. The multiplayer server will be that same crate, not a rewrite.

### Seven verbs

A cell's program talks to the kernel through a small, closed set of verbs:

| Verb | Meaning |
| --- | --- |
| **thrust** | Move. Costs mass (burned to the house). |
| **sense** | Look. Costs mass (burned to the house). |
| **absorb** | Take inert mass into yourself. Explicit. Touching does not eat. |
| **dump** | Leave inert mass in the dish (walls, shots, debris). Still in the box. |
| **attach** | Join cells into a body. |
| **split** | Divide a cell. Mass splits with it. |
| **cash out** | Mass leaves the box as money. |

Sleep is not a verb. Sleep is free.

`spend` is how thrust, sense, and any other compute/action bill the cell: mass moves from the cell to `house_burned` and is gone. `dump_matter` and `absorb_matter` move mass inside the box. They must conserve `total_mass()`.

## Skin

The skin is a static page that looks at the dish. It is not the game. This milestone's skin loads the kernel WASM, draws the dish on a canvas, and offers a small program editor (plus demo programs). It ships as plain HTML/CSS/JS plus `pkg/` from a public Cloud Storage bucket.

No CDN fonts. No random network calls. Relative paths. Serve over HTTP(S) so the browser can fetch the `.wasm` module (a `file://` open will not work).

## Later: server

Same kernel, authoritative, multiplayer. Not a second simulation. Guests still WASM, mass still fuel, conservation still load-bearing. How that process is hosted is a later, cheap decision. It is not Cloud Run, GKE, or a VM farm in this milestone.

## Mass accounting

The closed box is denominated in `Mass` (a `u64` newtype).

- `World::total_mass()` is living cells plus inert dumps. It is the mass still in the dish.
- `World::house_burned()` is mass destroyed by acting and computing. It only increases.
- `spawn_cell` is cash-in: new mass enters the box (bought).
- `spend` is the leak to the house.
- `dump_matter` / `absorb_matter` are internal transfers. They must not change `total_mass()`.
- Cash-out (later) is mass leaving the box as money, the inverse of spawn.

Tests in `crates/kernel` pin this down: conservation holds except `spend`, which increases `house_burned` by the same amount `total_mass()` drops.

## Why conservation matters

Mass is money. If the kernel mint or lose grams, the economy is a bug, not a game. Players cash out real value. The house burn is the only designed destruction: it is the cost of being awake and acting. Everything else is a transfer. That is why the first code in the repo is a mass ledger, not a renderer.
