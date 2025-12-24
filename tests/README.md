# Curl Test Suite

This folder contains curl scripts to smoke-test the API for health, auth, gating, and telemetry ingestion.

## Prerequisites
- Server running locally (Option A: Docker Compose or Option B: Local).
- Ensure `API_TOKEN` in your environment matches the server's configured token.
  - If you have a `.env` in the repo root, scripts will source it automatically.

## Quick Start

```bash
# From repo root, make scripts executable
chmod +x tests/*.sh

# Run all tests (sources .env if present)
bash tests/run-all.sh
```

## Individual Tests
Run any script individually:

```bash
# Health (no auth)
bash tests/health.sh

# 401 Unauthorized (missing token)
bash tests/unauthorized.sh

# 403 Forbidden (unknown vessel)
bash tests/forbidden.sh

# 200 OK (known vessel, mixed signals)
bash tests/ok_mixed.sh

# 200 OK with validSignals: 0 (type mismatches filtered)
bash tests/ok_zero_valid.sh
```

## Notes
- Default base URL is `http://localhost:8080`. Override with `BASE_URL` env var.
- Override token or other values via env vars when calling scripts:

```bash
API_TOKEN=your-token BASE_URL=http://localhost:8080 bash tests/ok_mixed.sh
```
