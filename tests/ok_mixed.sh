#!/usr/bin/env bash
set -euo pipefail
BASE_URL=${BASE_URL:-"http://localhost:8080"}
API_TOKEN=${API_TOKEN:-"seaker-telemetry-gateway-dev-token"}

curl -i -X POST "$BASE_URL/api/v1/telemetry" \
  -H "Authorization: Bearer $API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "vesselId":"1001",
    "timestampUTC":"2025-12-23T12:34:56Z",
    "signals":{"Signal_1":1,"Signal_70":123.4,"Signal_999":3.14}
  }'
