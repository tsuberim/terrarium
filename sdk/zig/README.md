# Terrarium Zig SDK

Write creatures in **Zig**, compile to WebAssembly, deploy to the sim.

**Recommended:** [Hello world in Replit](https://terrarium.mintlify.app/getting-started/replit) — no local install.

The server does **not** compile Zig. Deploy sends prebuilt WASM.

## Replit

1. [Import repo](https://replit.com/github.com/tsuberim/terrarium)
2. Paste `.env` from Terrarium (Code → Open in Replit)
3. Add `TERRARIUM_API_KEY` from in-game **Keys**
4. Edit `src/main.zig` → **Run**

## Local build

```bash
cd sdk/zig
zig build -Doptimize=ReleaseSmall
```

Output: `zig-out/bin/creature.wasm`

Requires [Zig](https://ziglang.org/download/) 0.13+.

## Deploy

```bash
./scripts/deploy.sh   # after setting env vars — see .env.example
```

Or drop `creature.wasm` in the Terrarium deploy dialog.

## Layout

| Path | Role |
|------|------|
| `src/terrarium.zig` | Host import bindings + helpers |
| `src/main.zig` | Your `tick` export |
| `build.zig` | `wasm32-freestanding` target |

## Host API

[docs](https://terrarium.mintlify.app/reference/host-abi) · imports use module name `"terrarium"`.

```zig
const tr = @import("terrarium.zig");

export fn tick() void {
    _ = tr.move_forward();
    tr.sleep_host();
}
```
