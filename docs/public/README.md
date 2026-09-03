# Public documentation (Mintlify)

**Players and external authors.** Published at [terrarium.mintlify.app](https://terrarium.mintlify.app).

Internal docs: [`../internal/`](../internal/) — not published here.

**Writing rules:** same as [internal/PRINCIPLES.md](../internal/PRINCIPLES.md) — shorter scope (players only), link out for depth.

---

## Purpose

| Public docs | Internal docs |
|-------------|---------------|
| How to play, author, deploy as a user | How to build, test, and ship the product |
| Short, approachable | Complete, precise, agent-oriented |
| Studio, SDK guides, concepts | PRD, workflow, architecture, tech-debt |
| No secrets, no QA hooks | Env vars, CI, `window.__TERRARIUM_E2E__` |

**Internal PRD wins** when content conflicts until public pages are updated.

---

## Local preview

```bash
npx mintlify dev --port 3333
# run from repo root; Mintlify root = docs/public
cd docs/public && npx mintlify dev
```

Requires [Mintlify CLI](https://mintlify.com/docs/development).

---

## Mintlify configuration

| File | Role |
|------|------|
| [docs.json](docs.json) | Nav, theme, OpenAPI tab |
| [openapi.json](openapi.json) | API Reference tab (may lag server; runtime spec is in `crates/server/`) |

**Dashboard setting:** point Mintlify GitHub integration at folder **`docs/public`**.

---

## Structure

```
getting-started/     onboarding flows
  studio.mdx           ← primary (in-game Rust)
  deploy-from-game.mdx
  replit.mdx           ← external Zig path

reference/           SDK & ABI
  rust-sdk.mdx
  zig-sdk.mdx
  host-abi.mdx
  wat.mdx              advanced
  rust-strategies.mdx  advanced

concepts/            short summaries (link to internal for depth)
  energy.mdx
  vision.mdx
```

---

## When to update public docs

| Trigger | Update |
|---------|--------|
| Studio UX change | `getting-started/studio.mdx`, maybe `deploy-from-game.mdx` |
| New SDK surface | `reference/*.mdx` |
| Economy rule players care about | `concepts/energy.mdx` + getting-started cost copy |
| API endpoint players call | `openapi.json` + Mintlify API tab |
| Principle / positioning change | `concepts/vision.mdx` after `internal/product/vision.md` |

**Do not update public docs for:** CI changes, QA testids, dev env vars, tech-debt, server thread model.

---

## Linking to internal docs

Public pages may link to GitHub for contributor depth:

```markdown
Full spec: [requirements.md](https://github.com/tsuberim/terrarium/blob/main/docs/internal/product/requirements.md)
```

Keep summaries self-contained — players should not *need* internal docs.

---

## Parent index

Structure diagram & e2e dev workflow: [`../README.md`](../README.md).
