# Pull requests

**Scope:** how to open and merge PRs. CI detail: [ci.md](ci.md). Doc rules: [../PRINCIPLES.md](../PRINCIPLES.md).

---

## Rules

1. **Docs before code** — same PR, always in sync ([PRINCIPLES § Docs-first](../PRINCIPLES.md#docs-first-development))
2. **Auto-merge** — every PR created with auto-merge enabled; merges when CI passes
3. **Title & description** — accurate, succinct, for someone not in the thread (see below)
4. **No force-push to `main`**

---

## Title & description

Write for an **outsider** — a reviewer or future you who wasn't in the work. No jargon dumps, no implementation trivia unless it matters.

**Title** — one line, what changed and why it matters. Not a commit list.

**Body** — bullets of the **main** changes only. Each bullet = one outcome or theme, not a file list.

```markdown
## Summary
- Split internal docs into product/workflow/engineering/qa/ops
- Add Playwright e2e driven by YAML scenarios in docs/internal/qa/
- Require auto-merge on all PRs

## Test plan
- [ ] npm run qa:all
```

Skip `--fill` if it produces commit-message noise; write title and body explicitly.

---

## Create a PR

```bash
gh pr create --title "..." --body "..." 
gh pr merge --auto
```

Or in one flow after push:

```bash
git push -u origin HEAD
gh pr create --fill
gh pr merge --auto
```

Auto-merge waits for the `ci.yml` gate (rust, frontend, docker, qa) to pass.

---

## PR body checklist

- [ ] Title and summary bullets reflect main changes for an outsider ([Title & description](#title--description))
- [ ] Docs updated before code
- [ ] [product/requirements.md](../product/requirements.md) matches implementation
- [ ] Other docs updated if scope changed (engineering, qa, ops, public)
- [ ] `./scripts/test.sh` and `npm run qa:all` pass locally

Full checklist: [../../README.md#pr-checklist-docs--code-in-sync](../../README.md#pr-checklist-docs--code-in-sync).

---

## After merge

`main` triggers `deploy.yml` (when `DEPLOY_ENABLED=true`). See [../ops/deploy.md](../ops/deploy.md).
