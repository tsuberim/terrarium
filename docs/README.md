# Documentation

Terrarium has **two doc systems** with different audiences, rules, and publish paths. They are related but not interchangeable.

| System | Path | Audience | Published to |
|--------|------|----------|--------------|
| **Internal** | [`internal/`](internal/) | Contributors, agents, ops | GitHub repo only |
| **Public** | [`public/`](public/) | Players, external authors | [terrarium.mintlify.app](https://terrarium.mintlify.app) |

**Rule:** Docs lead, code follows. Both stay in sync in every PR. See [internal/PRINCIPLES.md](internal/PRINCIPLES.md).

---

## How the two systems interact

```
                    ┌─────────────────────────────────────┐
                    │         INTERNAL (source)           │
                    │                                     │
                    │  product/requirements.md  ◄── PRD   │
                    │  product/vision.md        ◄── why     │
                    │  workflow/                ◄── how     │
                    │  engineering/, qa/, ops/              │
                    └──────────────┬──────────────────────┘
                                   │
              decide + implement   │   distill (player language)
                                   ▼
                    ┌─────────────────────────────────────┐
                    │         PUBLIC (published)          │
                    │                                     │
                    │  public/getting-started/*           │
                    │  public/reference/*                 │
                    │  public/concepts/*                  │
                    │  public/openapi.json (API tab)      │
                    └──────────────┬──────────────────────┘
                                   │
                                   ▼
                         terrarium.mintlify.app
                                   │
                                   ▼
                              Players / Replit
```

### What flows where

| Internal change | Update internal | Update public |
|-----------------|-----------------|---------------|
| New feature / acceptance criteria | `internal/product/requirements.md` | Relevant `public/getting-started/` or `public/reference/` page |
| New env var, script, CI job | `internal/workflow/README.md` | Only if players need it (rare) |
| Sim rule / economy math | `internal/engineering/sim/*`, PRD §6 | `public/concepts/energy.mdx` |
| Host ABI change | `internal/engineering/sim/host-abi.md` | `public/reference/host-abi.mdx` |
| Strategic shift | `internal/product/vision.md` → PRD | `public/concepts/vision.mdx` |
| Engineering shortcut | `internal/engineering/tech-debt.md` | Don't expose unless user-visible |

**Never** put secrets, QA hooks, agent instructions, or infra credentials in public docs.

---

## Internal structure

→ Full index: [`internal/README.md`](internal/README.md)

```
docs/internal/
├── README.md
├── PRINCIPLES.md
├── product/
│   ├── requirements.md      ← PRD
│   └── vision.md
├── workflow/
│   ├── README.md            ← dev loop
│   ├── setup.md
│   ├── ci.md
│   ├── prs.md               ← auto-merge policy
│   └── troubleshooting.md
├── engineering/
│   ├── principles.md        ← how we change code
│   ├── architecture.md
│   ├── testing.md
│   ├── tech-debt.md
│   └── sim/
│       ├── energy-budget.md
│       └── host-abi.md
├── qa/
│   ├── README.md
│   └── scenarios/
└── ops/
    ├── deploy.md
    ├── environments.md
    └── secrets.md
```

Also: [`AGENTS.md`](../AGENTS.md), [`.cursor/skills/`](../.cursor/skills/), [`sdk/`](../sdk/).

---

## Public structure

→ Maintenance guide: [`public/README.md`](public/README.md)

```
docs/public/
├── README.md
├── docs.json                ← Mintlify nav
├── index.mdx
├── getting-started/
│   ├── studio.mdx           ← primary path
│   ├── deploy-from-game.mdx
│   └── replit.mdx
├── reference/
│   ├── rust-sdk.mdx
│   ├── zig-sdk.mdx
│   ├── host-abi.mdx
│   ├── wat.mdx
│   └── rust-strategies.mdx
├── concepts/
│   ├── energy.mdx
│   └── vision.mdx
└── openapi.json             ← Mintlify API tab (not server runtime spec)
```

Mintlify root path: **`docs/public`** (configure in Mintlify dashboard if not already).

---

## End-to-end dev workflow

Docs-first — write specs before code, keep in sync through merge:

```
┌──────────────────────────────────────────────────────────────────────────┐
│ 1. DOCS FIRST                                                             │
│    product/requirements.md — add/change requirement, set status             │
│    engineering / qa / ops / public — as needed for the change             │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│ 2. CODE                                                                   │
│    Implement to match docs                                                │
│    Diverged? → update docs in same PR (never merge out of sync)           │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│ 3. VERIFY                                                                 │
│    ./scripts/test.sh · dev.sh · npm run qa:all                            │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│ 4. PR → auto-merge when CI passes ([workflow/prs.md](internal/workflow/prs.md)) │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│ 5. DEPLOY                                                                 │
│    main → deploy.yml (when DEPLOY_ENABLED=true)                           │
│    Detail: internal/ops/deploy.md                                         │
└──────────────────────────────────────────────────────────────────────────┘
```

### Quick commands

```bash
./scripts/setup-dev.sh   # once
./scripts/dev.sh         # terminal 1
npm run qa:all           # terminal 2
```

Detail: [`internal/workflow/README.md`](internal/workflow/README.md).

---

## Entry points by role

| Role | Start here |
|------|------------|
| **AI agent** | [`AGENTS.md`](../AGENTS.md) → [`internal/product/requirements.md`](internal/product/requirements.md) → [`internal/workflow/README.md`](internal/workflow/README.md) |
| **Contributor** | [`internal/workflow/README.md`](internal/workflow/README.md) |
| **Ops / deploy** | [`internal/ops/deploy.md`](internal/ops/deploy.md) |
| **Player / external author** | [terrarium.mintlify.app](https://terrarium.mintlify.app) — source in [`public/`](public/) |

---

## PR checklist (docs + code in sync)

- [ ] Docs updated **before** code ([PRINCIPLES.md](internal/PRINCIPLES.md))
- [ ] [product/requirements.md](internal/product/requirements.md) — requirement + status matches implementation
- [ ] Other internal docs updated if scope changed (workflow, engineering, qa, ops)
- [ ] [public/](public/) updated if player-visible
- [ ] Code verifies against doc acceptance criteria
- [ ] PR title & summary bullets are outsider-friendly ([workflow/prs.md](internal/workflow/prs.md))
- [ ] PR opened with auto-merge ([workflow/prs.md](internal/workflow/prs.md))
