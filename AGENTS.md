# AGENTS.md

Guide for AI agents working in this repository.

## Docs-first (required)

**Update docs before code. Keep both in sync in the same PR.**

1. Edit [product/requirements.md](docs/internal/product/requirements.md) (and eng/qa/ops/public as needed)
2. Implement to match
3. If code diverges → update docs before merge
4. Verify: `./scripts/test.sh`, `npm run qa:all`

Full rules: [docs/internal/PRINCIPLES.md](docs/internal/PRINCIPLES.md)

---

## Start here

| Doc | Scope |
|-----|-------|
| [docs/README.md](docs/README.md) | Doc map, e2e workflow |
| [docs/internal/PRINCIPLES.md](docs/internal/PRINCIPLES.md) | Writing + docs-first rules |
| [docs/internal/engineering/principles.md](docs/internal/engineering/principles.md) | How we change code |
| [docs/internal/product/requirements.md](docs/internal/product/requirements.md) | PRD |
| [docs/internal/workflow/README.md](docs/internal/workflow/README.md) | Dev loop |

Public: [terrarium.mintlify.app](https://terrarium.mintlify.app) · [`docs/public/`](docs/public/)

---

## Default dev loop

**Keep `./scripts/dev.sh` running** for the whole session when the user is involved. Start it if down; leave it up between tasks.

```bash
./scripts/setup-dev.sh   # once
./scripts/dev.sh         # terminal 1 — keep running
npm run qa:all           # terminal 2
```

Eng principles: [docs/internal/engineering/principles.md](docs/internal/engineering/principles.md)

---

## Key paths

| Area | Path |
|------|------|
| UI / Studio | `apps/skin/src/` |
| API | `crates/server/` |
| Sim | `crates/sim/` |
| QA scenarios | `docs/internal/qa/scenarios/` |
| Playwright | `apps/skin/e2e/` |

---

## QA hooks (`VITE_QA_MODE=true`)

- `window.__TERRARIUM_QA__.getState()`
- `data-testid="qa-*"`
- `document.body.dataset.qaReady === "true"`

---

## Skills

| Skill | Use |
|-------|-----|
| [`.cursor/skills/browser-qa/SKILL.md`](.cursor/skills/browser-qa/SKILL.md) | Cursor browser QA |

---

## Commits & PRs

Do not commit unless asked. Every PR: [auto-merge, babysit until merged](docs/internal/workflow/prs.md). CI must pass: rust, frontend, docker, qa.
