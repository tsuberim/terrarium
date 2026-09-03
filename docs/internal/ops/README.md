# Ops

**Scope:** prod deploy, GCP/Firebase setup, secret names. Not local dev — see [../workflow/setup.md](../workflow/setup.md).

**Prod URL:** https://terrarium-506917.web.app · **GCP:** `terrarium-506917` (`us-central1`)

---

## Files

| Doc | Scope |
|-----|-------|
| [deploy.md](deploy.md) | Prod architecture, deploy, prod troubleshooting |
| [environments.md](environments.md) | One-time GCP/Firebase bootstrap |
| [secrets.md](secrets.md) | Env var names, GitHub secrets |

---

## CI / deploy flow

PR tests → merge to `main` → deploy workflow → `smoke-prod.sh`.

Detail: [../workflow/ci.md](../workflow/ci.md), [deploy.md](deploy.md).
