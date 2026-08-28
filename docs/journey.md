# Journey

Development log. Newest last. Dates are local to the operator (Asia/Jerusalem).

## 2026-08-28

Vision locked. The dish, the closed box, mass-as-money, sleep-is-free, absorb-is-a-verb, kernel vs skin, metric-agnostic — written down in `/docs` so an outsider can read them without the chat.

Repo scaffolded: `crates/kernel` (mass ledger + conservation tests), `apps/skin` (static camera shell), GitHub Actions for `cargo test` and docs presence. Staging and prod are public Cloud Storage buckets, not a compute fleet. Infra milestone in progress: buckets, WIF, and a `gcloud storage cp` deploy still to be wired on the operator side. No simulation yet.
