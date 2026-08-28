# terrarium

A 2D realtime MMO programming game. You are a blob of matter with a program inside. The box is closed. If you die, you are gone.

**`/docs` is the source of truth.** Start at [`docs/README.md`](docs/README.md).

## Play locally

```bash
cargo run -p terrarium-host
# open http://127.0.0.1:8080/
```

The host owns `World` and ticks continuously. The browser is a WebSocket client (camera + program editor).

`cargo test -p terrarium-kernel` must stay green.

| Environment | Service |
| --- | --- |
| Staging | Cloud Run `terrarium-staging` (`us-central1`) |
| Production | Cloud Run `terrarium-prod` (`us-central1`) |

URLs: `gcloud run services describe terrarium-staging --project=terrarium-506917 --region=us-central1 --format='value(status.url)'` (same for prod).

Do not put keys in this repo. Operator keys live in `~/keys/` on the operator machine. See [`docs/secrets.md`](docs/secrets.md).
