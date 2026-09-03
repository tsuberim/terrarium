# Pull requests

**Scope:** how to open and merge PRs. CI detail: [ci.md](ci.md). Doc rules: [../PRINCIPLES.md](../PRINCIPLES.md).

---

## Rules

1. **Docs before code** — same PR, always in sync ([PRINCIPLES § Docs-first](../PRINCIPLES.md#docs-first-development))
2. **Auto-merge** — every PR created with auto-merge enabled; merges when CI passes
3. **Babysit** — whoever opens the PR owns it until merged (see below)
4. **Title & description** — accurate, succinct, for someone not in the thread (see below)
5. **No force-push to `main`**

---

## Babysitting

Opening a PR means you **own it until it's on `main`**.

1. Enable auto-merge (`gh pr merge --auto`)
2. **Monitor CI in the background** — poll checks without blocking the conversation
3. **Fix failures** — push fixes, re-enable auto-merge if needed
4. **Confirm merge** when green

```bash
gh pr checks 44 --watch          # optional; or poll between other work
gh run list --branch "$(git branch --show-current)" --limit 3
```

Don't hand off a red PR. If CI is still running when the session ends, note status and what's left.

Write for an **outsider** — a reviewer or future you who wasn't in the work. No jargon dumps, no implementation trivia unless it matters.

**Title** — one line, what changed and why it matters. Not a commit list.

**Body** — bullets of the **main** changes only. Each bullet = one outcome or theme, not a file list.

```markdown
## Summary
- Split internal docs into product/workflow/engineering/qa/ops
- Add Playwright e2e driven by YAML scenarios in docs/internal/qa/
- Require auto-merge on all PRs

## Test plan
- [ ] npm run test:integration
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

Auto-merge waits for the `ci.yml` gate (rust, frontend, docker, smoke, e2e) to pass.

---

## PR body checklist

- [ ] Title and summary bullets reflect main changes for an outsider ([Title & description](#title--description))
- [ ] Docs updated before code
- [ ] [product/requirements.md](../product/requirements.md) matches implementation
- [ ] Other docs updated if scope changed (engineering, qa, ops, public)
- [ ] `./scripts/test.sh` and `npm run test:integration` pass locally

Full checklist: [../../README.md#pr-checklist-docs--code-in-sync](../../README.md#pr-checklist-docs--code-in-sync).

---

## After merge

`main` triggers `deploy.yml` (when `DEPLOY_ENABLED=true`). See [../ops/deploy.md](../ops/deploy.md).
