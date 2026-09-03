# Product documentation

**Scope:** how to maintain the PRD. Requirements live in [requirements.md](requirements.md).

---

## Files

| Doc | Scope |
|-----|-------|
| [requirements.md](requirements.md) | PRD — behavior & acceptance criteria |
| [vision.md](vision.md) | Principles & direction |

Eng debt: [../engineering/tech-debt.md](../engineering/tech-debt.md). Public sync: [../../public/README.md](../../public/README.md).

---

## Status labels

Use in `requirements.md`:

| Label | Meaning |
|-------|---------|
| **shipped** | In prod / local dev as described |
| **dev-only** | Works locally; prod may differ |
| **in progress** | Partially implemented |
| **planned** | Agreed direction, not built |

---

## When to update

**Before coding:**

- New feature → add requirement to `requirements.md` (`in progress`)
- Behavior change → update PRD + affected docs first
- Strategic shift → `vision.md` first

**Before merge:**

- Set final status (`shipped`, `dev-only`, etc.)
- Player-visible → [public/](../../public/) after PRD
- Code and docs must agree
