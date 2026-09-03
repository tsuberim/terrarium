# Public documentation (Mintlify)

**Players and external authors.** Published at [terrarium.mintlify.app](https://terrarium.mintlify.app).

Internal docs: [`../internal/`](../internal/) — not published here.

**Writing rules:** [internal/PRINCIPLES.md](../internal/PRINCIPLES.md) — shorter scope (players only).

---

## Mintlify configuration

| File | Role |
|------|------|
| [docs.json](docs.json) | Nav, theme, OpenAPI tab |
| [openapi.json](openapi.json) | API Reference tab — synced from `crates/server/src/openapi.json` via `./scripts/sync-openapi.sh` |

**Dashboard:** GitHub folder **`docs/public`**.

---

## Structure

```
getting-started/     studio.mdx · deploy-from-game.mdx
reference/           rust-sdk.mdx · host-abi.mdx
concepts/            energy.mdx · vision.mdx
openapi.json         API tab
```

---

## When to update

| Trigger | Update |
|---------|--------|
| Studio UX | `getting-started/studio.mdx` |
| SDK / ABI | `reference/*.mdx` |
| Player-facing economy | `concepts/energy.mdx` |
| API change | `crates/server/src/openapi.json` then `./scripts/sync-openapi.sh` |

Do **not** update for CI, e2e hooks, or internal-only workflow.

Parent index: [`../README.md`](../README.md).
