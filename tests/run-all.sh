#!/usr/bin/env bash
set -euo pipefail

# Source .env if present in repo root
if [ -f .env ]; then
  set -a
  # shellcheck disable=SC1091
  . ./.env
  set +a
fi

BASE_URL=${BASE_URL:-"http://localhost:8080"}

run() {
  echo "\n==== $1 ===="
  bash "$1"
}

run tests/health.sh
run tests/unauthorized.sh
run tests/forbidden.sh
run tests/ok_mixed.sh
run tests/ok_zero_valid.sh

echo "\nAll tests completed."
