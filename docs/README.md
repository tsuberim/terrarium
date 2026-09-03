# Documentation

Two doc systems — different audiences, same PR sync rule. See [PRINCIPLES.md](internal/PRINCIPLES.md).

| System | Path | Audience | Published |
|--------|------|----------|-----------|
| **Internal** | [`internal/`](internal/) | Contributors, agents | GitHub only |
| **Public** | [`public/`](public/) | Players, authors | [terrarium.mintlify.app](https://terrarium.mintlify.app) |

```
internal/  product · workflow · engineering · qa · ops
    │ distill player-visible changes
    ▼
public/    getting-started · reference · concepts · openapi.json
    ▼
terrarium.mintlify.app
```

**What flows where:** feature → `internal/product/requirements.md` + eng/qa/ops as needed → relevant `public/` page if player-visible. Never put secrets or e2e hooks in public docs.

---

## Entry points

| Role | Start |
|------|-------|
| **AI agent** | [AGENTS.md](../AGENTS.md) |
| **Contributor** | [internal/workflow/README.md](internal/workflow/README.md) |
| **Ops** | [internal/ops/deploy.md](internal/ops/deploy.md) |
| **Player** | [terrarium.mintlify.app](https://terrarium.mintlify.app) |

Internal index: [internal/README.md](internal/README.md). Public maintenance: [public/README.md](public/README.md).

---

## Dev loop

Docs first → code → `./scripts/test.sh` + `npm run test:integration` → PR with auto-merge. Detail: [internal/workflow/README.md](internal/workflow/README.md).
