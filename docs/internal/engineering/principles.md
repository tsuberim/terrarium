# Engineering principles

**Scope:** how we change code. Doc rules: [../PRINCIPLES.md](../PRINCIPLES.md). Architecture: [architecture.md](architecture.md).

---

## Core rules

1. **Judge, don't inherit** — don't assume the current code is correct or optimal. Read it, decide if it's good enough; improve when it isn't.

2. **Architecture over compatibility** — prefer clear structure and good abstractions over preserving every old path. Breaking changes are OK when they make the system better.

3. **Flag regressions** — if a refactor breaks behavior, tests, or UX, call it out explicitly so we can choose: fix forward, accept the tradeoff, or revert.

4. **Leave it better** — every touch should improve readability, structure, or correctness. No drive-by unrelated churn; no leaving known mess behind when you're already there.

5. **Simplify** — compress, remove redundancy, improve efficiency — but **readability wins**. Clever one-liners that obscure intent are not a win.

6. **Complexity = incoherence** — tangled code usually means the concept isn't settled yet. Don't paper over it with more branches. Either:
   - **Define what we already know** — write it down (PRD, eng doc, types, names) so code can stay dumb, or
   - **Discuss and hash it out** — real ambiguity needs a decision together, not a local workaround.

---

## In practice

| Situation | Do |
|-----------|-----|
| Touching messy code in your path | Clean the slice you're changing |
| Old pattern blocks a better design | Propose/replace; note what breaks |
| Test fails after refactor | Say whether it's a real regression or outdated test |
| "Works but ugly" | Fix if cost is small; add to [tech-debt.md](tech-debt.md) if not |
| Hard to explain or refactor | Stop adding code — define the concept in docs or raise it for discussion |

Docs-first still applies: behavior changes → PRD + docs before code ([../PRINCIPLES.md](../PRINCIPLES.md)).
