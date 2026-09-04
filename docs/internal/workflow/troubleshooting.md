# Troubleshooting (local)

**Scope:** local dev problems. Prod issues: [../ops/deploy.md](../ops/deploy.md#prod-troubleshooting).

---

| Symptom | Fix |
|---------|-----|
| `:5173` refused | `./scripts/dev.sh` |
| Compile fails `expected item, found keyword let` | `./scripts/dev-stop.sh && ./scripts/dev.sh` (stale worker) |
| `body_wrap: true` on worker health | Restart dev — worker should report `body_wrap: false` |
| Sign-in fails | `VITE_USE_AUTH_EMULATOR=true`; check `:9099` |
| Studio clicks do nothing | Open Studio first (`e2e-hud-studio`) |
| `UNIQUE constraint failed: creatures.x,y` | `rm data/terrarium.db && ./scripts/dev.sh` |
| QA bridge undefined | `VITE_E2E_HOOKS=true` in `.env.local`, restart Vite |
| Playwright duplicate `e2e-world-map` | Use main map testid only |

Setup detail: [setup.md](setup.md). QA pitfalls: [../qa/README.md](../qa/README.md).
