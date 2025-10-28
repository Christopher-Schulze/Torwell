#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

echo "[run_all] Running Rust test suite"
pushd src-tauri >/dev/null
cargo test --all-targets
popd >/dev/null

echo "[run_all] Running UI test suite"
npm test -- --runInBand

echo "[run_all] Verifying TLS compliance"
node scripts/tests/check_tls.js
