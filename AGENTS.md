# Agent rules

Repo-critical rules for coding agents. Personal working notes and feedback live in operator MEMORIES.md, updated from Slack `#performance-review`.

## Git

Repo: [github.com/tsuberim/terrarium](https://github.com/tsuberim/terrarium). **Push to `main`. No PRs unless explicitly asked.**

Cloud agent env already has `gh` logged in and `git` remote wired — no PAT hunt.

## Ship

CI must be green. QA staging (manual or tests) before every release. Don't ship blind.

## Docs

`/docs` is source of truth. Read before coding. Don't contradict vision or architecture without talking it through.

## Secrets

Keys live in `~/keys/` only (operator machine). Never print, commit, or paste tokens. See [`docs/secrets.md`](docs/secrets.md).

## GCP

Project `terrarium-506917`. Watch cost. Staging/prod stay cheap GCS buckets unless we explicitly decide otherwise.
