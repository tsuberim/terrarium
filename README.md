# terrarium

A 2D realtime MMO programming game. You are a blob of matter with a program inside. The box is closed. If you die, you are gone.

**`/docs` is the source of truth.** Start at [`docs/README.md`](docs/README.md).

## Play locally

```bash
# optional, if you changed the kernel:
./scripts/build-wasm.sh

# serve the static skin (required — browsers won't load WASM from file://)
python3 -m http.server 8080 --directory apps/skin
# open http://127.0.0.1:8080/
```

`cargo test --manifest-path crates/kernel/Cargo.toml` must stay green.

| Environment | URL |
| --- | --- |
| Staging | https://storage.googleapis.com/terrarium-506917-staging/index.html |
| Production | https://storage.googleapis.com/terrarium-506917-prod/index.html |

Staging and production are public Cloud Storage buckets serving the static skin over HTTPS. No Cloud Run, no containers, no load balancers for this milestone.

Do not put keys in this repo. Operator keys live in `~/keys/` on the operator machine. See [`docs/secrets.md`](docs/secrets.md).
