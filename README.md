# Terrarium

Persistent 2D programmable-creature simulation. See [docs/vision.md](docs/vision.md).

## Local dev

One-time setup:

```bash
chmod +x scripts/*.sh
./scripts/setup-dev.sh
```

Start API + frontend together (watch mode, foreground):

```bash
./scripts/dev.sh
```

Background watch mode (preferred):

```bash
./scripts/dev-bg.sh    # → http://localhost:5173
./scripts/dev-stop.sh  # stop
```

Open **http://localhost:5173** — Vite proxies `/api` to the Rust server on `:8080`.

Optional: `cargo install cargo-watch` for API auto-reload on file changes.

**Firebase auth locally:** ensure `localhost` is in [Authorized domains](https://console.firebase.google.com/project/terrarium-506917/authentication/settings) and Google sign-in is enabled under Sign-in method.

Run separately if needed:

```bash
./scripts/run-server.sh          # API only
cd apps/skin && npm run dev      # frontend only
```

## DevOps

Full reference: [docs/devops.md](docs/devops.md)

- [Environments & one-time setup](docs/environments.md)
- [Secrets (local + CI)](docs/secrets.md)

Push to `main` runs CI tests and deploys Cloud Run + Firebase Hosting.

Live: https://terrarium-506917.web.app
