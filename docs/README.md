# Docs

This folder is the source of truth for terrarium: intent, architecture, environments, and current state. Read these before the code. They are written for someone who has never seen the project.

| Doc | What it is |
| --- | --- |
| [vision.md](vision.md) | What the game is, and the rules that do not move |
| [architecture.md](architecture.md) | Kernel, native host, skin client, mass accounting, the seven verbs |
| [environments.md](environments.md) | CI, Cloud Run staging/prod, GCP project, how deploys stay cheap |
| [secrets.md](secrets.md) | Keys live in `~/keys/` only. Never the repo, never copies in GitHub secrets |
| [journey.md](journey.md) | Development log |
| [current-state.md](current-state.md) | Honest snapshot of what exists right now |

The public repo is [github.com/tsuberim/terrarium](https://github.com/tsuberim/terrarium). The GCP project is `terrarium-506917`.
