# 🚀 Telemetry Ingestor

> Production-grade telemetry ingestion service built with Rust, Axum, and PostgreSQL

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![PostgreSQL](https://img.shields.io/badge/postgres-%23316192.svg?style=flat&logo=postgresql&logoColor=white)](https://www.postgresql.org/)
[![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=flat&logo=docker&logoColor=white)](https://www.docker.com/)

---

## 📋 Table of Contents

- [Features](#-features)
- [Architecture](#-architecture)
- [API Reference](#-api-reference)
- [Getting Started](#-getting-started)
- [Database Schema](#-database-schema)
- [Configuration](#-configuration)
- [Testing](#-testing)
- [Troubleshooting](#-troubleshooting)

---

## ✨ Features

- **High Performance**: Built on Tokio async runtime with Axum 0.7 framework
- **Type-Safe Database**: PostgreSQL with sqlx for compile-time safety
- **Secure Authentication**: Bearer token API key authentication
- **Comprehensive Validation**: Strict signal type checking and range validation
- **Production Observability**: Structured logging with `tracing` and metrics tracking
- **Container Ready**: Full Docker and docker-compose support
- **Graceful Shutdown**: Proper signal handling for clean termination
- **Signal Classification**: Automatic routing to `main_raw` or `filtered_raw` based on validation

---

## 🏗 Architecture

### Tech Stack

| Component | Technology |
|-----------|------------|
| **Runtime** | Rust (Tokio async) |
| **HTTP Framework** | Axum 0.7 |
| **Database** | PostgreSQL 16 |
| **SQL Driver** | sqlx (async) |
| **Authentication** | Bearer Token |
| **Observability** | tracing + metrics |
| **Containerization** | Docker + Compose |

### System Architecture

```mermaid
graph TB
    Client[HTTP Client] -->|POST /api/v1/telemetry| Router[Axum Router]
    Router --> Auth[Auth Middleware]
    Auth --> Handler[Telemetry Handler]
    Handler --> VR[(vessel_register_table)]
    Handler --> SR[Signal Registry<br/>in-memory]
    Handler --> MR[(main_raw<br/>valid signals)]
    Handler --> FR[(filtered_raw<br/>invalid signals)]
    Handler --> MET[(server_metrics)]
    
    style Client fill:#e1f5ff
    style Router fill:#fff4e1
    style Auth fill:#fff4e1
    style Handler fill:#fff4e1
    style VR fill:#e8f5e9
    style SR fill:#f3e5f5
    style MR fill:#e8f5e9
    style FR fill:#ffebee
    style MET fill:#e8f5e9
```

### Request Sequence

```
Request Sequence

Client ----> Router ----> Handler ----> Postgres

- Parse / validate JSON
- SELECT EXISTS(vessel) ----> Postgres ----> (true/false)
- Loop signals
  - valid ----> INSERT main_raw
  - invalid/unknown ----> INSERT filtered_raw (with reason)
- INSERT server_metrics
- Handler ----> Client: 200 OK {counts, timings}
```Flow

```📁 Project Structure

```
src/
├── 🚀 main.rs              # Application bootstrap & graceful shutdown
├── 🔧 app.rs               # Router setup & shared state
├── ⚙️  config.rs            # Environment-driven configuration
├── 📊 db/
│   ├── mod.rs
│   └── postgres.rs         # Database access layer (sqlx)
├── 🔒 middleware/
│   ├── mod.rs
│  **Request Received**: Client sends `POST /api/v1/telemetry` with JSON payload
2. **Authentication**: Bearer token validated against `API_TOKEN`
3. **Vessel Validation**: Vessel ID checked in `vessel_register_table` for active status
4. **Signal Validation**: Each signal validated against registry:
   - **Digital** (`Signal_1` - `Signal_50`): Must be `0` or `1` (integer)
   - **Analog** (`Signal_51` - `Signal_200`): Must be float in range `1.0` - `65535.0`
5. **Storage Routing**:
   - ✅ **Valid signals** → `main_raw` table
   - ❌ **Invalid/unknown signals** → `filtered_raw` table (with reason)
6. **Metrics Recording**: Request timings stored in `server_metrics`
7. **Response**: JSON summary with counts and performance metrics
     ▼
┌─────────────────┐
│  Parse JSON     │──── ✗ Invalid Format ──► 400 Bad Request
└────┬────────────┘
     │ ✓ Valid JSON
     ▼
┌─────────────────┐
│  Check Vessel   │──── ✗ Unknown/Inactive ──► 403 Forbidden
└────┬────────────┘
     │ ✓ Active Vessel
     ▼
┌─────────────────┐
│ Validate Each   │
│    Signal       │
└────┬────────────┘
     │
     ├──► Valid Signal ────► INSERT main_raw
     │
     └──► Invalid Signal ──► INSERT filtered_raw (with reason)
     
     ▼
┌─────────────────┐
│ Record Metrics  │──► INSERT server_metrics
└────┬────────────┘
     │
     ▼
┌─────────────────┐
---

## 📡 API Reference

### Telemetry Ingestion
```
Authorization: Bearer <API_TOKEN>
Content-Type: application/json
```

**Request Body:**
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

**Response:**
- **Endpoint:** `POST /api/v1/telemetry`
- **Headers:** `Authorization: Bearer <API_TOKEN>`
- **Body (example):**

```json
{
  "vesselId": "1001",
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

**Status Codes:**

| Code | Description |
|------|-------------|
| `200` | Request processed successfully |
| `400` | Invalid request format or timestamp |
| `401` | Missing or invalid authentication token |
| `403` | Unknown or inactive vessel |
| `500` | Internal server error |

---

### Health Check

**Endpoint:** `GET /healthz`
Schema defined in [db/init.sql](db/init.sql)

### Tables

| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `vessel_register_table` | Active vessel registry | `vessel_id`, `name`, `is_active` |
| `signal_register_table` | Signal type definitions | `signal_name`, `signal_type` |
| `main_raw` | Valid telemetry data | `vessel_id`, `signal_name`, `signal_value` |
| `filtered_raw` | Invalid/filtered data | `vessel_id`, `signal_name`, `reason` |
| `server_metrics` | Request performance metrics | `vessel_id`, `validation_ms`, `ingestion_ms` |

### Seed Data

---

## 🚀 Getting Started

### Prerequisites

- **Rust** 1.70+ (install via [rustup](https://rustup.rs/))
- **PostgreSQL** 16+ 
- **Docker** & **Docker Compose** (optional, for containerized deployment)

### Quick Start (Local Development

**Response (Degraded):**
```json
{
  "status": "degraded",
  "db": "down"
}
```

**Example:**Ms": 5,
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
---

### Option A: Docker Compose (Recommended)

#### 1️⃣ Start All Services
```bash
docker compose up --build -d
```

#### 2️⃣ Verify Deploymentes:

```bash
brew services start postgresql@16
```

3. Configure environment:

```bash
cp .env.example .env
# adjust .env if needed; defaults work for local services
```
---

### Option B: Hybrid Setup

#### 1️⃣ Start PostgreSQL Container
```bash
docker run -d --name telemetry-db \
  -p 5432:5432 \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=telemetry \
  postgres:16
```

#### 2️⃣ Initialize Database
```bash
psql -U postgres -h 127.0.0.1 -d telemetry -f db/init.sql
```

#### 3️⃣ Configure & Run
```bash
cp .env.example .env
# Ensure DATABASE_URL=postgres://postgres:postgres@localhost:5432/telemetry
cargo run
```

---
| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://postgres:postgres@localhost:5432/telemetry` |
| `API_TOKEN` | Bearer token for authentication | `seaker-telemetry-gateway-dev-token` |
---

## 📝 Implementation Notes

### Validation
- **Timestamps**: Parsed using RFC3339 format via `chrono`
- **Signal Types**: Strict validation by type and range
- **Digital Signals**: Only integers `0` or `1` accepted
- **Analog Signals**: Only floats in range `1.0` - `65535.0` accepted
- **Type Mismatches**: Strings, booleans, or wrong numeric types → `filtered_raw` with `type_mismatch` reason
- **Range Violations**: Out-of-range values → `filtered_raw` with `out_of_range` reason

### Data Layer
- **SQLx**: Runtime queries for flexibility without compile-time DB dependency
- **Connection Pool**: Managed async database connections
- **Signal Registry**: Loaded on startup and cached in-memory
- **Vessel Validation**: Real-time checks against PostgreSQL

### Storage Model
- **Row-per-Signal**: Narrow table schema for optimal query performance
- **Dual Tables**: Valid signals in `main_raw`, invalid in `filtered_raw`
- **Metrics Tracking**: Request timings stored per vessel

### Operational
- **Graceful Shutdown**: Handles `SIGINT` (`Ctrl+C`) and `SIGTERM`
- **Structured Logging**: Full request tracing via `tracing` crate
- **Health Checks**: Database connectivity monitoring

---
---

## 🧪🔧 **Connection Tuning**: Optimize `sqlx::Pool` settings for load

### Reliability
- 🔄 **Connection Retries**: Implement automatic reconnection logic
- 📊 **Monitoring**: Expand metrics collection (Prometheus/OpenTelemetry)
- 🔍 **Distributed Tracing**: Add trace IDs for request correlation
### Manual Testing Suite

Test all major scenarios using curl:

#### ✅ Test 1: Health Check (No Auth)y**: Duplicate requests may create duplicate records
- 🔨 **Manual Migrations**: Schema changes require manual SQL execution
- 🗄️ **Single Database**: No built-in sharding or read replicas
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

## Scaling Considerations

- Batch inserts (COPY or multi-row INSERT) for higher throughput.
- Apply backpressure limits via Tower layers.
- Use `sqlx::Pool` tuning and connection retries.

## Failure Modes

- `401` Unauthorized when Bearer token mismatches.
- `403` when vessel is unknown/inactive.
- `400` for invalid `timestampUTC`.
- `500` for DB errors (summarized without leaking details).

## Known Limitations

- No idempotency keys; duplicates possible if retried.
- No schema migrations tool; init via SQL file.

## Troubleshooting

- 401 Unauthorized: Ensure header is exactly `Authorization: Bearer <API_TOKEN>` and matches your `.env`.
- DB connection errors: Confirm Postgres is running and `DATABASE_URL` is correct. When using Docker, connect via `localhost` mapping.
- Schema missing: Re-run [db/init.sql](db/init.sql) against your target database.

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