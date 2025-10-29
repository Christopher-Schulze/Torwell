#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT/src-tauri"

echo "[run_concurrency] Running scheduler loom model"
RUSTFLAGS="--cfg loom" cargo test --lib core::executor::loom_tests::metrics_batch_is_linearizable -- --nocapture

if command -v cargo-miri >/dev/null 2>&1; then
  echo "[run_concurrency] Running scheduler loom model under Miri"
  cargo miri test --lib core::executor::loom_tests::metrics_batch_is_linearizable -- --nocapture
else
  echo "[run_concurrency] cargo-miri not installed; skipping Miri run" >&2
fi
