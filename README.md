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
