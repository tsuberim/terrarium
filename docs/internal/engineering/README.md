# Engineering

**Scope:** how the system works — server, sim, ABI, debt. Not product UX requirements or dev commands.

**Product behavior:** [../product/requirements.md](../product/requirements.md)

---

## Touching X → read Y

| You're changing… | Read |
|------------------|------|
| Any code | [principles.md](principles.md) |
| WebSocket, server threads, fan-out | [architecture.md](architecture.md) |
| Tick rules, economy, food mint | [sim/energy-budget.md](sim/energy-budget.md), PRD §4–6 |
| Host syscalls, WASM ABI | [sim/host-abi.md](sim/host-abi.md) |
| External control WS, API key attach | [external-control.md](external-control.md) |
| Unit/integration tests | [testing.md](testing.md) |
| Known shortcuts / scale limits | [tech-debt.md](tech-debt.md) |

---

## Files

| Doc | Scope |
|-----|-------|
| [principles.md](principles.md) | How we change code |
| [architecture.md](architecture.md) | Server, WS protocol, scale path |
| [testing.md](testing.md) | Rust/server test layers |
| [tech-debt.md](tech-debt.md) | Eng debt backlog |
| [sim/energy-budget.md](sim/energy-budget.md) | Economy math |
| [sim/host-abi.md](sim/host-abi.md) | WASM host ABI |
| [external-control.md](external-control.md) | API-key control WS, signal/recv relay |

---

## Key code paths

| Area | Path |
|------|------|
| UI | `apps/skin/src/` |
| API server | `crates/server/` |
| Sim | `crates/sim/` |
| Compile worker | `services/compile-worker/` |
| Rust SDK | `sdk/rust/terrarium-sdk/` |

QA automation: [../qa/README.md](../qa/README.md) — not duplicated here.
