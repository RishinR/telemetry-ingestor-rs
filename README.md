# Telemetry Ingestor (Rust + Axum + Postgres)

Production-grade telemetry ingestion service designed for interview evaluation and real-world backend scenarios.

## Architecture Overview

- **Runtime:** Rust (Tokio async)
- **HTTP:** Axum 0.7
- **Database:** PostgreSQL (sqlx)
- **Auth:** Bearer token (simple API key)
- **Observability:** `tracing` logs + server metrics table
- **Containerization:** Dockerfile + docker-compose

## System Architecture

```
System Architecture

Client (HTTP Client)
    
                    ┌─────────────────────────┐
                    │   Telemetry Client      │
                    │  (Vessel / Simulator)   │
                    └───────────┬─────────────┘
                                │
                                │ HTTP POST /api/v1/telemetry
                                │ Authorization: Bearer <token>
                                ▼
        ┌─────────────────────────────────────────────────┐
        │         Telemetry Ingestor (Rust + Axum)        │
        │                                                 │
        │  ┌──────────────┐   ┌────────────────────────┐  │
        │  │ Auth Layer   │──▶│  Telemetry Handler     │  │
        │  │ (Bearer)     │   │                        │  │
        │  └──────────────┘   │  - JSON validation     │  │
        │                     │  - Vessel gating       │  │
        │                     │  - Signal validation   │  │
        │                     │  - Metrics capture     │  │
        │                     └───────────┬────────────┘  │
        │                                 │               │
        │       In-memory Signal Registry │               │
        │       (loaded on startup)       │               │
        │                                 │               │
        └─────────────────────────────────┼───────────────┘
                                          │
                                          ▼
                    ┌────────────────────────────────────┐
                    │            PostgreSQL              │
                    │                                    │
                    │  vessel_register_table             │
                    │  signal_register_table             │
                    │  main_raw        (valid signals)   │
                    │  filtered_raw    (invalid signals) │
                    │  server_metrics  (observability)   │
                    └────────────────────────────────────┘

```

## Data Flow

```
Data Flow

Client
  │
  │ POST /api/v1/telemetry
  ▼
Auth Middleware
  │
  ├─ Invalid token ───────────▶ 401 Unauthorized
  │
  ▼
Telemetry Handler
  │
  ├─ Unknown vessel ─────────▶ 403 PermissionDenied
  │
  ▼
Signal Validation
  │
  ├─ Valid signals ──────────▶ main_raw
  │
  ├─ Invalid signals ────────▶ filtered_raw
  │
  ▼
Metrics Capture
  │
  ▼
200 OK (summary)

```

### Request Sequence

```
Request Sequence

Client           API Server                 PostgreSQL
  │                  │                          │
  │ POST /telemetry  │                          │
  │─────────────────▶│                          │
  │                  │ Validate Bearer token    │
  │                  │                          │
  │                  │ Check vesselId           │
  │                  │────────────────────────▶ │
  │                  │ ◀──────── exists?        │
  │                  │                          │
  │                  │ Validate signals         │
  │                  │ (in-memory registry)     │
  │                  │                          │
  │                  │ INSERT valid signals     │
  │                  │────────────────────────▶ │
  │                  │                          │
  │                  │ INSERT invalid signals   │
  │                  │────────────────────────▶ │
  │                  │                          │
  │                  │ INSERT metrics           │
  │                  │────────────────────────▶ │
  │                  │                          │
  │ 200 OK + summary │                          │
  │◀─────────────────│                          │

```

### Components & Files
- Router/State: [src/app.rs](src/app.rs)
- Config: [src/config.rs](src/config.rs)
- Middleware (auth): [src/middleware/auth.rs](src/middleware/auth.rs) layered via router
- Handler: [src/routes/telemetry.rs](src/routes/telemetry.rs)
- DB access (sqlx): [src/db/postgres.rs](src/db/postgres.rs)

- Models: [src/models/telemetry.rs](src/models/telemetry.rs)
- Bootstrap & graceful shutdown: [src/main.rs](src/main.rs)

### Data Flow

1. Client sends `POST /api/v1/telemetry` with JSON payload
2. Bearer token checked against `API_TOKEN`
3. Vessel ID validated in `vessel_register_table`
4. Signals validated against registry:
   - `Signal_1..Signal_50` → digital (0 | 1)
   - `Signal_51..Signal_200` → analog (1.0 ..= 65535.0)
5. Valid signals → `main_raw`
6. Invalid/unknown signals → `filtered_raw` with reason
7. Metrics recorded in `server_metrics`
8. JSON summary returned

## API

- **Endpoint:** `POST /api/v1/telemetry`
- **Headers:** `Authorization: Bearer <API_TOKEN>`
- **Body (example):**

```json
{
  "vesselId": "1001",
  "timestampUTC": "2025-12-23T12:34:56Z",
  "signals": {
    "Signal_1": 1,
    "Signal_70": 123.4,
    "Signal_999": 3.14
  }
}
```

- **Response (example):**

```json
{
  "ok": true,
  "vesselId": "1001",
  "validSignals": 2,
  "validationMs": 3,
  "ingestionMs": 5,
  "totalMs": 9
}
```

- **Health Check:**
  - Endpoint: `GET /healthz`
  - Auth: none (public)
  - 200 OK: `{ "status": "ok", "db": "up" }`
  - 503 Service Unavailable: `{ "status": "degraded", "db": "down" }`
  - Example:

```bash
curl -i http://localhost:8080/healthz
```

## Database Schema

Defined in [db/init.sql](db/init.sql):

- `vessel_register_table(vessel_id, name, is_active)`
- `signal_register_table(signal_name, signal_type)`
- `main_raw(id, vessel_id, timestamp_utc, signal_name, signal_value, created_at)`
- `filtered_raw(id, vessel_id, timestamp_utc, signal_name, signal_value, reason, created_at)`
- `server_metrics(id, vessel_id, validation_ms, ingestion_ms, total_ms, created_at)`

Signals are seeded: `Signal_1..50` digital, `Signal_51..200` analog. Vessels `1001`, `1002` seeded active.

## Running Locally

### Quick Start (Local, no Docker for app)

1. Install prerequisites (macOS):

```bash
brew install postgresql@16
rustup update
```

2. Start Postgres:

```bash
brew services start postgresql@16
```

3. Configure environment:

```bash
cp .env.example .env
# adjust .env if needed; defaults work for local services
```

4. Create database and apply schema/seed:

```bash
createdb telemetry || true
psql -U postgres -d telemetry -f db/init.sql
```

5. Run the server:

```bash
cargo build
cargo run
```

6. Smoke test:

```bash
curl -i -X POST http://localhost:8080/api/v1/telemetry \
  -H "Authorization: Bearer seaker-telemetry-gateway-dev-token" \
  -H "Content-Type: application/json" \
  -d '{
    "vesselId": "1001",
    "timestampUTC": "2025-12-23T12:34:56Z",
    "signals": {"Signal_1": 1, "Signal_70": 123.4, "Signal_999": 3.14}
  }'
```

### Option A: Docker Compose (recommended)

1. Build and start services

```bash
docker compose up --build -d
```

2. Test the API

```bash
curl -X POST http://localhost:8080/api/v1/telemetry \
  -H "Authorization: Bearer seaker-telemetry-gateway-dev-token" \
  -H "Content-Type: application/json" \
  -d '{
    "vesselId": "1001",
    "timestampUTC": "2025-12-23T12:34:56Z",
    "signals": {"Signal_1": 1, "Signal_70": 123.4, "Signal_999": 3.14}
  }'
```

### Option B: App locally, services via Docker

Run Postgres in a container, app locally:

```bash
docker run -d --name telemetry-db -p 5432:5432 \
  -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=telemetry \
  postgres:16

psql -U postgres -h 127.0.0.1 -d telemetry -f db/init.sql

cp .env.example .env
# Ensure DATABASE_URL points at localhost

cargo run
```

## Configuration

Environment variables (see `.env.example`):

- `DATABASE_URL` → e.g., `postgres://postgres:postgres@localhost:5432/telemetry`
- `API_TOKEN` → Bearer token expected by server
- `PORT` → default `8080`

## Implementation Notes

- **Validation:** Timestamp parsed RFC3339 via `chrono`; signals validated by type and range.
- **SQLx:** Runtime queries (`query`, `bind`) to avoid compile-time DB requirement; still async and safe.
- **Caching:** In-process only. Signal registry is loaded from Postgres on startup and kept in memory. Vessel activity is checked directly against Postgres.
- **Epoch vs Timestamp:** `epochUTC` (if present) uses basic format validation only; not strictly matched to `timestampUTC`.
- **Storage Model:** Narrow row-per-signal schema (Postgres) aligning with SQL guidance.
- **Metrics:** Timing captured around validation, ingestion, and total request.
- **Graceful Shutdown:** Handles `Ctrl+C` and `SIGTERM` on Unix.
 - **Strict Signal Typing:** Digital signals accept only JSON numbers that are integers `0` or `1`. Analog signals accept only JSON numbers that are floats in the range `1.0..=65535.0`. Non-numeric types (strings, booleans) or mismatched numeric types (e.g., integer for analog) are not ingested; they are written to `filtered_raw` with reason `type_mismatch`. Out-of-range numeric values are written with reason `out_of_range`.

## Failure Modes

- `401` Unauthorized when Bearer token mismatches.
- `403` when vessel is unknown/inactive.
- `400` for invalid `timestampUTC`.
- `500` for DB errors (summarized without leaking details).

## Testing

Smoke-test the service with curl. These cover health, auth, gating, and ingest.

1) Health (no auth):

```bash
curl -i http://localhost:8080/healthz
```

2) 401 Unauthorized (missing token):

```bash
curl -i -X POST http://localhost:8080/api/v1/telemetry \
  -H "Content-Type: application/json" \
  -d '{"vesselId":"1001","timestampUTC":"2025-12-23T12:34:56Z","signals":{"Signal_1":1}}'
```

3) 403 Forbidden (unknown vessel):

```bash
curl -i -X POST http://localhost:8080/api/v1/telemetry \
  -H "Authorization: Bearer seaker-telemetry-gateway-dev-token" \
  -H "Content-Type: application/json" \
  -d '{"vesselId":"unknown-xyz","timestampUTC":"2025-12-23T12:34:56Z","signals":{"Signal_1":1}}'
```

4) 200 OK (known vessel, mixed signals):

```bash
curl -i -X POST http://localhost:8080/api/v1/telemetry \
  -H "Authorization: Bearer seaker-telemetry-gateway-dev-token" \
  -H "Content-Type: application/json" \
  -d '{"vesselId":"1001","timestampUTC":"2025-12-23T12:34:56Z","signals":{"Signal_1":1,"Signal_70":123.4,"Signal_999":3.14}}'
```

5) 200 OK with `validSignals: 0` (type mismatches filtered):

```bash
curl -i -X POST http://localhost:8080/api/v1/telemetry \
  -H "Authorization: Bearer seaker-telemetry-gateway-dev-token" \
  -H "Content-Type: application/json" \
  -d '{"vesselId":"1001","timestampUTC":"2025-12-23T12:34:56Z","signals":{"Signal_1":true,"Signal_70":"123.4"}}'
```

These mismatches are stored in `filtered_raw` with `type_mismatch`.

## Project Structure

```
src/
 ├── main.rs          # bootstrap, tracing, shutdown
 ├── app.rs           # router + shared state
 ├── config.rs        # env-driven config
 ├── db/              # Postgres functions (sqlx)
 ├── models/          # request models
 └── routes/          # telemetry handler
```
