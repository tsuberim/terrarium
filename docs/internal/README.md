# Internal documentation

**Contributors and agents only.** Not published to Mintlify.

Public docs: [`../public/`](../public/) → [terrarium.mintlify.app](https://terrarium.mintlify.app).

Doc map: [`../README.md`](../README.md). **Rules:** [`PRINCIPLES.md`](PRINCIPLES.md) — docs before code, always in sync.

---

## Work order

```
docs → code → sync → verify → merge
```

See [PRINCIPLES.md § Docs-first](PRINCIPLES.md#docs-first-development).

---

## Read order

| Step | Doc |
|------|-----|
| 1 | [product/requirements.md](product/requirements.md) — what to build |
| 2 | [workflow/README.md](workflow/README.md) — how to work |
| 3 | One area below for your change |

Agents: [`AGENTS.md`](../../AGENTS.md).

---

## Structure

```
internal/
├── product/          what & why (PRD, vision)
├── workflow/         how to work (setup, ci, troubleshooting)
├── engineering/      how it works (architecture, sim, tech-debt)
├── qa/               test hooks & scenarios
└── ops/              prod deploy & secrets
```

---

## Folders

| Folder | Scope |
|--------|-------|
| [product/](product/) | PRD, vision |
| [workflow/](workflow/) | Dev loop, setup, CI, PRs |
| [engineering/](engineering/) | Architecture, sim, tests, tech-debt |
| [qa/](qa/) | QA framework, scenarios |
| [ops/](ops/) | Prod deploy, GCP, secrets |

Also: [`AGENTS.md`](../../AGENTS.md), [`.cursor/skills/`](../../.cursor/skills/), [`sdk/`](../../sdk/).

---

## Public sync

Player-visible change → PRD first → distill to [`../public/`](../public/).
