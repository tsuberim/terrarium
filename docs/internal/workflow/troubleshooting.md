# Troubleshooting (local)

**Scope:** local dev problems. Prod issues: [../ops/deploy.md](../ops/deploy.md#prod-troubleshooting).

---

| Symptom | Fix |
|---------|-----|
| `:5173` refused | `./scripts/dev.sh` |
| Compile fails `expected item, found keyword let` | `./scripts/dev-stop.sh && ./scripts/dev.sh` (stale worker) |
| `body_wrap: false` on worker health | Restart dev |
| Sign-in fails | `VITE_USE_AUTH_EMULATOR=true`; check `:9099` |
| Studio clicks do nothing | Open Studio first (`qa-hud-studio`) |
| `UNIQUE constraint failed: creatures.x,y` | `rm data/terrarium.db && ./scripts/dev.sh` |
| QA bridge undefined | `VITE_QA_MODE=true` in `.env.local`, restart Vite |
| Playwright duplicate `qa-world-map` | Use main map testid only |

Setup detail: [setup.md](setup.md). QA pitfalls: [../qa/README.md](../qa/README.md).
