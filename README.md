# terrarium

A 2D realtime MMO programming game. You are a blob of matter with a program inside. The box is closed. If you die, you are gone.

**`/docs` is the source of truth.** Start at [`docs/README.md`](docs/README.md).

| Environment | URL |
| --- | --- |
| Staging | https://storage.googleapis.com/terrarium-506917-staging/index.html |
| Production | https://storage.googleapis.com/terrarium-506917-prod/index.html |

Staging and production are public Cloud Storage buckets serving the static skin over HTTPS. No Cloud Run, no containers, no load balancers for this milestone.

Do not put keys in this repo. Operator keys live in `~/keys/` on the operator machine. See [`docs/secrets.md`](docs/secrets.md).
