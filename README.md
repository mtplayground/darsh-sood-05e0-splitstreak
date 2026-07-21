# SplitStreak

SplitStreak is a workout logging PWA with a React SPA frontend and a Rust/Axum
backend API.

## Project Layout

- `frontend/` - Vite, React, and TypeScript single-page app.
- `backend/` - Rust Axum HTTP API.
- `.plan` - Architecture plan for the full issue backlog.

## Local Development

Install JavaScript dependencies:

```bash
npm install
```

Create a local environment file from the checked-in example:

```bash
cp .env.example .env
```

Run the frontend and backend side by side:

```bash
npm run dev
```

The backend listens on `0.0.0.0:8080` by default and exposes:

- `GET /api/health`
- `GET /health`

The Vite dev server proxies `/api/*` to the backend.

The backend requires PostgreSQL via `DATABASE_URL`. In this workspace:

```bash
export DATABASE_URL=$(cat /workspace/.database_url)
```

Run migrations manually:

```bash
npm run backend:migrate
```

Build both projects:

```bash
npm run build
```

Run consistency checks:

```bash
npm run typecheck
npm run lint
npm run format:check
```

## Self-Hosted Deployment

SplitStreak can be deployed without Docker and without a CI/CD pipeline. The
release artifact is a Rust API binary plus the built Vite frontend. Runtime
configuration comes only from environment variables.

Build the artifact on the target host or on a compatible build machine:

```bash
npm run self-hosted:build
```

The default output is `dist/self-hosted/`:

- `bin/splitstreak-api` - backend API and static frontend server.
- `frontend/` - built SPA assets copied from `frontend/dist`.
- `.env.example` - environment variable reference.
- `run-self-hosted.sh` - minimal runtime wrapper for the artifact.

Run it with environment variables:

```bash
cd dist/self-hosted
export DATABASE_URL=postgres://user:password@host:5432/splitstreak
export SELF_URL=https://splitstreak.example.com
export MCTAI_AUTH_URL=https://auth.mctai.app
export MCTAI_AUTH_APP_TOKEN=app_darsh-sood-05e0-splitstreak-7150a4
export MCTAI_AUTH_JWKS_URL=https://auth.mctai.app/.well-known/jwks.json
sh ./run-self-hosted.sh
```

The server listens on `HOST` and `PORT`, defaulting to `0.0.0.0:8080`.
`STATIC_DIR` defaults to the artifact's `frontend/` directory so browser routes
fall back to `index.html`. Set `STATIC_DIR` yourself only when serving a
different frontend directory.

Required production variables:

- `DATABASE_URL` - PostgreSQL connection string.
- `SELF_URL` - public frontend URL used for auth redirects.
- `MCTAI_AUTH_URL`, `MCTAI_AUTH_APP_TOKEN`, `MCTAI_AUTH_JWKS_URL` - Ideavibes
  auth service configuration.

Optional production variables:

- `HOST` - bind host, default `0.0.0.0`.
- `PORT` - bind port, default `8080`.
- `STATIC_DIR` - directory containing `index.html` and built assets.
- `DATABASE_MAX_CONNECTIONS` - Postgres pool size, default `5`.
- `MCTAI_EMAIL_URL`, `MCTAI_EMAIL_APP_TOKEN` - Ideavibes email sending.
- `RUST_LOG` - tracing filter, for example `splitstreak_api=info,tower_http=info`.

The binary runs embedded SQLx migrations at startup before serving traffic. To
run migrations and exit:

```bash
./bin/splitstreak-api --migrate-only
```

Terminate with `Ctrl-C` or your process manager's normal signal. Put TLS,
compression, and long-running restart policy in a host-level reverse proxy or
service manager.
