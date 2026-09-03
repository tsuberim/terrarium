# Doc writing principles

Every internal and public doc follows these rules.

---

## Docs-first development

**All work starts in docs, then code.** Docs and code stay in sync — same PR, no exceptions.

```
1. Update docs (what + acceptance criteria + status)
2. Implement to match docs
3. If implementation diverges → update docs in the same PR before merge
4. Verify (tests, qa)
5. Merge — docs and code describe the same thing
```

| Situation | Docs first |
|-----------|------------|
| New feature | Add/update PRD requirement (`in progress` → `shipped`) |
| Behavior change | Update PRD + affected eng/qa/ops docs, then code |
| Bug fix (spec was wrong) | Fix doc if spec was wrong, then code |
| Bug fix (code was wrong) | Code fix; doc unchanged unless behavior clarifies |
| Refactor (no behavior change) | No PRD change; eng docs if structure matters |
| Player-visible | PRD → `public/*.mdx` → code |

**Never merge** code that contradicts docs, or docs that describe unbuilt behavior without an explicit status (`planned`, `in progress`).

---

## Writing rules

1. **Short.** Say it once. Cut filler.
2. **Plain words.** No jargon unless the reader already uses it.
3. **One job per doc.** Split or link if scope grows.
4. **No duplication.** One canonical home per fact.
5. **Scope visible.** First paragraph: audience + what this file does *not* cover.

---

## Scope by doc

| Doc | Owns | Does not own |
|-----|------|--------------|
| `product/requirements.md` | Shipped behavior, acceptance criteria | Dev commands, sim math |
| `product/vision.md` | Why, principles, non-goals | Acceptance criteria |
| `workflow/` | Commands, loop, local troubleshooting | Prod infra, sim rules |
| `workflow/prs.md` | PR policy, auto-merge, babysitting, title/body | CI job definitions |
| `engineering/` | Architecture, sim, ABI, tech-debt | Product UX requirements |
| `engineering/principles.md` | How we change code | Doc writing rules |
| `ops/` | Prod deploy, GCP, secrets | Local dev |
| `qa/` | Test hooks, scenarios, runners | Product requirements |
| `public/*.mdx` | Player how-to | Internal QA, CI, agents |
| `.cursor/skills/` | Agent procedure | Specs — link to docs |

---

## Before you add text

- Can this live in the **canonical** doc instead?
- Already written somewhere? → link.
- Will the reader know **when to stop reading**?

---

## Public vs internal

Same rules. Public = shorter scope (players only). Summarize; link to GitHub for depth.
