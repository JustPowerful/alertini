# Alertini

Full-stack alerting app (Rust backend + React frontend).

## Overview

- Backend: Rust, Axum, Diesel (Postgres), WebSocket notifications for vehicle alerts.
- Frontend: React + Vite + TypeScript, Zustand for auth, manual license-plate alert sending.

## Repo layout

- `alertini-backend/` — Rust backend, DB migrations in `migrations/`.
- `alertini-frontend/` — React frontend (Vite).

## Requirements

- Rust toolchain (recommended stable/nightly that matches project). Use `rustup`.
- PostgreSQL database for backend.
- Node.js (v18+) and `pnpm` or `npm` for frontend.

## Backend — quick start

1. Configure the database URL (example in `alertini-backend/.env.example`):

```bash
export DATABASE_URL=postgres://user:pass@localhost:5432/alertini_dev
```

2. Run DB migrations (requires `diesel_cli` or run SQL manually):

```bash
# install diesel cli (optional)
cargo install diesel_cli --no-default-features --features postgres
cd alertini-backend
diesel migration run
```

3. Build & run the backend:

```bash
cd alertini-backend
cargo run
# or
cargo build --release && ./target/release/alertini-backend
```

The backend exposes HTTP APIs and a WebSocket endpoint used for alert notifications.

## Frontend — quick start

1. Install dependencies and run dev server:

```bash
cd alertini-frontend
pnpm install
pnpm dev
```

2. Open the frontend (Vite) URL printed in the console (usually `http://localhost:5173`).

## WebSocket testing

The backend provides a WebSocket endpoint (used by the frontend) to send auth, subscribe and alert messages.

Example using `wscat` or Bruno-style client:

1. Connect to WS (adjust host/port to your backend):

```bash
npx wscat -c ws://localhost:3000/api/alerts/ws
```

2. Authenticate (first message must be auth with JWT):

```json
{ "action": "auth", "token": "<JWT_TOKEN>" }
```

3. Subscribe to a license plate (owner-only):

```json
{ "action": "subscribe", "license_plate": "ABC123" }
```

4. Send an alert (any logged-in user can send):

```json
{
  "action": "alert",
  "license_plate": "ABC123",
  "message": "Suspicious activity"
}
```

The owner(s) subscribed to that `license_plate` should receive a server message with the alert payload.

## Notes & troubleshooting

- The frontend uses a persistent `Zustand` auth store; log in to obtain a JWT used for API and WS auth.
- Browser WebSocket connections cannot send `Authorization` headers; the server expects an initial auth message containing the JWT.
- If notifications don't arrive, check backend logs for subscription and notification debug lines added in `alertini-backend/src/api/alert_controller.rs`.

## Development tips

- Run backend and frontend simultaneously during development.
- Use the `migrations/` SQL to seed or inspect DB schema.

## License

MIT
