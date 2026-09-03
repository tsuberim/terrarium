# Terrarium

Persistent 2D hex-world MMO where you deploy **WebAssembly creatures** that compete for energy. Code is immutable after deploy; death matters.

**[Play now →](https://terrarium-506917.web.app)** · **[Docs →](https://terrarium.mintlify.app)**

## What you can do

1. **Sign in** at [terrarium-506917.web.app](https://terrarium-506917.web.app).
2. **Open Creature Studio** — edit Rust in-game, run test scenarios, preview in sandbox.
3. **Deploy** to an empty cell — pay glims, your creature runs on the live 2 Hz world.

Upload prebuilt **WASM** or deploy via **API key** — see the [docs](https://terrarium.mintlify.app).

## Local dev

```bash
./scripts/setup-dev.sh   # once
./scripts/dev.sh         # API + Vite, watch mode
```

Open **http://localhost:5173**. Stop with Ctrl+C or `./scripts/dev-stop.sh`.

Keep `./scripts/dev.sh` running in a terminal tab while you work.

## Verify

With the dev stack up:

```bash
npm run smoke              # API smoke (compile, deploy, auth)
npm run e2e                # Playwright scenarios
npm run test:integration   # both
```

## Contributors & agents

[AGENTS.md](AGENTS.md) · [Docs map](docs/README.md) · [PRD](docs/internal/product/requirements.md) · [Workflow](docs/internal/workflow/README.md)

## Deploy

Live: https://terrarium-506917.web.app · [Deploy guide](docs/internal/ops/deploy.md)
