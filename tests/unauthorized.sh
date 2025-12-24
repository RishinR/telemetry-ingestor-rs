#!/usr/bin/env bash
set -euo pipefail
BASE_URL=${BASE_URL:-"http://localhost:8080"}

curl -i -X POST "$BASE_URL/api/v1/telemetry" \
  -H "Content-Type: application/json" \
  -d '{"vesselId":"1001","timestampUTC":"2025-12-23T12:34:56Z","signals":{"Signal_1":1}}'
