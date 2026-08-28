# Product specs

Concrete product behaviour — what the game should look and feel like. Vision and architecture stay in their own docs; this file is the checklist for skin and UX work.

## How specs work

Slack `#product-requests` is intake. Drop a bullet there when you want something changed. Once we agree it is real work, the spec lands here — not in chat threads.

Read this file before coding a product change. When something ships or starts, update its status so the doc stays honest.

Status tags:

- **shipped** — in main, matches the spec below
- **in progress** — someone is building it
- **requested** — agreed direction, not done yet

## Fullscreen retro camera

**Status: shipped**

The skin is a fullscreen retro camera on the sim. No chrome, no dashboard.

- Pixelated look — chunky framebuffer, nearest-neighbor upscale (`image-rendering: pixelated`), CRT scanlines
- Minimalist — world fills the viewport; almost nothing else on screen
- No stats / HUD — no tick counter, mass totals, house burned, FPS, or other overlays on the raw sim
- Hideable program overlay — wander / chase / sit demos and the paste-a-program editor stay available, but tuck away so writing a creature program does not break the fullscreen feel

Kernel rules unchanged. The camera gets prettier; the box does not.

## Wrapping / toroidal open world

**Status: requested** (implementation on a separate branch)

Not a petri dish. The world wraps like a torus:

- Move off the right edge → pop in on the left (same y)
- Move off the left edge → pop in on the right
- Same for top and bottom

No hard walls that stop you at the rim. No circular "dish" boundary. Open world feel with finite wrap-around space.

Mass conservation and kernel verbs stay as in vision — this is geometry and camera framing, not new economy rules.
