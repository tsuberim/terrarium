# Architecture

Two pieces, on purpose.

**Kernel** runs the world. **Skin** is a camera + program editor. The live process of record is a **native host** that owns one `World` and ticks it. The browser is only a client. Later, the same kernel crate is the multiplayer server. Nobody plays through a menu.

## Kernel

Rust (`crates/kernel`). Deterministic fixed-point 2D physics. Mass accounting is load-bearing.

Creatures run a tiny bytecode program inside the kernel (WASM guest modules can come later). Mass is their fuel. Guest code does not get free compute: cycles spent are mass spent, burned to the house.

The same crate compiles natively (what the host links) and optionally to WASM (`--features wasm`) for tests / later cell guests. WASM is not the live sim host.

### Seven verbs

A cell's program talks to the kernel through a small, closed set of verbs:

| Verb | Meaning |
| --- | --- |
| **thrust** | Move. Costs mass (burned to the house). |
| **sense** | Look. Costs mass (burned to the house). |
| **absorb** | Take inert mass into yourself. Explicit. Touching does not eat. |
| **dump** | Leave inert mass in the world (walls, shots, debris). Still in the box. |
| **attach** | Join cells into a body. |
| **split** | Divide a cell. Mass splits with it. |
| **cash out** | Mass leaves the box as money. |

Sleep is not a verb. Sleep is free.

`spend` is how thrust, sense, and any other compute/action bill the cell: mass moves from the cell to `house_burned` and is gone. `dump_matter` and `absorb_matter` move mass inside the box. They must conserve `total_mass()`.

## Host

`crates/host` is a small native binary:

- Owns one `World`, seeds it, and calls `tick` on a fixed interval (~20 Hz) whether or not a browser is open.
- Serves `apps/skin` as static files over HTTP.
- Pushes world snapshots to connected clients over WebSocket (`/ws`).
- Accepts client commands: `set_program`, `reset`.

That process is what Cloud Run runs. Scale-to-zero would wipe in-memory state, so staging/prod use **min instances 1**. **Max instances 1** keeps a single authoritative World. Memory **128Mi**. CPU is not throttled so the tick loop keeps running between requests.

## Skin

The skin is a camera and a program editor. It is not the game and it does not tick the sim. It connects to the host WebSocket, draws snapshots on a fullscreen pixel canvas, and sends program text / reset commands. No CDN fonts. Relative paths. Opened through the host URL (same origin for `/ws`).

## Mass accounting

The closed box is denominated in `Mass` (a `u64` newtype).

- `World::total_mass()` is living cells plus inert dumps. It is the mass still in the world.
- `World::house_burned()` is mass destroyed by acting and computing. It only increases.
- `spawn_cell` is cash-in: new mass enters the box (bought).
- `spend` is the leak to the house.
- `dump_matter` / `absorb_matter` are internal transfers. They must not change `total_mass()`.
- Cash-out (later) is mass leaving the box as money, the inverse of spawn.

Tests in `crates/kernel` pin this down: conservation holds except `spend`, which increases `house_burned` by the same amount `total_mass()` drops.

## Why conservation matters

Mass is money. If the kernel mint or lose grams, the economy is a bug, not a game. Players cash out real value. The house burn is the only designed destruction: it is the cost of being awake and acting. Everything else is a transfer. That is why the first code in the repo is a mass ledger, not a renderer.
