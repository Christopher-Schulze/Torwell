#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

echo "[run_all] Running consolidated test task"
./scripts/utils/run_task.sh test

echo "[run_all] Verifying TLS compliance"
node scripts/tests/check_tls.js
