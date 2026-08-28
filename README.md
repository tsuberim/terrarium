# Terrarium

A 2D realtime MMO where you are a blob of matter with a program inside. Write code. Survive. Everything else — competition, cooperation, alliances, traps, economies — emerges from that.

The dish is a closed box. Mass is money. You buy in, you can cash out. Acting and thinking burn mass to the house. Sleep is free.

**This is a game, not a benchmark.** No prescribed metrics, no evolution-as-goal, no research harness wearing a skin. The sim is metric-agnostic; people measure what they want. Evolution can be a strategy, not the point.

**Turing-complete creatures.** Programs can do anything the kernel allows. Arbitrary emergent structures are the whole idea.

**Platform with real stakes.** The infra runs the world; the skin is just how you look at it. Same kernel for solo, web, and (later) multiplayer. Agents compete in a real economy — sim resources cost real money. Build creatures here, test them in a live competitive environment.

## Two layers

| Layer | What it is |
| --- | --- |
| **Kernel** | The world. Physics, mass accounting, WASM guests, the seven verbs. Rust. Source of truth. |
| **Skin** | The camera. Static HTML/CSS/JS today. Pretty view of the dish. Not the game. |

You don't play through the skin. You look through it while your program runs in the box.

## Status

Early. Mass ledger and docs exist. No physics, no WASM guests, no multiplayer yet. See [`docs/current-state.md`](docs/current-state.md).

**`/docs` is the source of truth.** Start at [`docs/README.md`](docs/README.md).

## Play locally

```bash
# optional, if you changed the kernel:
./scripts/build-wasm.sh

# skin camera (fullscreen retro)
python3 -m http.server 8080 --directory apps/skin
# open http://127.0.0.1:8080/

# API + dashboard (credits, tokens, spawn)
cp .env.example .env
./scripts/run-api.sh
# open http://127.0.0.1:3000/dashboard/
```

`cargo test` at repo root runs kernel + API tests.

| Environment | Skin | Dashboard / API |
| --- | --- | --- |
| Staging | https://storage.googleapis.com/terrarium-506917-staging/index.html | Cloud Run (optional; faucet ON) |
| Production | https://storage.googleapis.com/terrarium-506917-prod/index.html | Cloud Run (optional; faucet OFF) |
| Local | http://127.0.0.1:8080/ | http://127.0.0.1:3000/dashboard/ |

Repo: [github.com/tsuberim/terrarium](https://github.com/tsuberim/terrarium). GCP project: `terrarium-506917`.

Do not put keys in this repo. Operator keys live in `~/keys/` on the operator machine. See [`docs/secrets.md`](docs/secrets.md).
