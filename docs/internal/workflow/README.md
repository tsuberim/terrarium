# Development workflow

How to work on Terrarium. **Scope:** commands, loop, pointers.

**Rule:** [docs-first](../PRINCIPLES.md#docs-first-development) — update docs before code, keep both in sync in the same PR.

**Agents:** [AGENTS.md](../../AGENTS.md) → [doc map](../../README.md) → here.

---

## Quick reference

| Task | Command |
|------|---------|
| First-time setup | `./scripts/setup-dev.sh` |
| Daily dev | `./scripts/dev.sh` |
| Stop | Ctrl+C or `./scripts/dev-stop.sh` |
| Fast check (no stack) | `./scripts/test.sh` |
| Full local QA | `npm run qa:all` |

**Local URL:** http://localhost:5173

---

## Standard loop

**Dev server:** while you're actively working with an agent, keep `./scripts/dev.sh` running. Start it if down; don't stop it when a task ends unless asked.

```
1. DOCS — update product/requirements.md (+ eng/qa/ops/public as needed)
2. CODE — implement to match docs
3. SYNC — if code diverged, update docs in same PR
4. VERIFY — ./scripts/test.sh, dev.sh, npm run qa:all
5. MERGE — docs and code agree → PR with auto-merge ([prs.md](prs.md))
```

Run verification yourself. Eng principles: [../engineering/principles.md](../engineering/principles.md).

---

## Deep dives

| Topic | Doc |
|-------|-----|
| Setup & dev stack | [setup.md](setup.md) |
| CI & GitHub Actions | [ci.md](ci.md) |
| Pull requests & auto-merge | [prs.md](prs.md) |
| Engineering principles | [../engineering/principles.md](../engineering/principles.md) |
| Local problems | [troubleshooting.md](troubleshooting.md) |
| Unit/integration tests | [../engineering/testing.md](../engineering/testing.md) |
| QA hooks & e2e | [../qa/README.md](../qa/README.md) |
| Prod deploy | [../ops/deploy.md](../ops/deploy.md) |
| Doc rules | [../PRINCIPLES.md](../PRINCIPLES.md) |

---

## Scripts (common)

| Script | Use |
|--------|-----|
| `setup-dev.sh` | One-time bootstrap |
| `dev.sh` / `dev-stop.sh` | Local stack |
| `test.sh` | fmt, clippy, unit, lint, build |
| `qa-smoke.sh` / `qa-e2e.sh` / `ci-e2e.sh` | QA layers |

---

## Which doc to update first

| Change | Update before coding |
|--------|----------------------|
| Behavior / acceptance criteria | [product/requirements.md](../product/requirements.md) |
| Commands, env, CI | this folder |
| Sim / server / debt | [../engineering/](../engineering/) |
| QA hooks / scenarios | [../qa/](../qa/) |
| Prod / secrets | [../ops/](../ops/) |
| Player-visible | [../../public/](../../public/) after PRD |
