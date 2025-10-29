#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TARGET_TEST=${1:-parallel_metrics_benchmark}
OUT_DIR="$ROOT/src-tauri/target/memory-profiles"
OUT_FILE="$OUT_DIR/massif-${TARGET_TEST}-$(date +%Y%m%d-%H%M%S).out"

if ! command -v valgrind >/dev/null 2>&1; then
  echo "[run_massif] valgrind is not available" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"

pushd "$ROOT/src-tauri" >/dev/null
cargo test --release --no-run
valgrind \
  --tool=massif \
  --trace-children=yes \
  --massif-out-file="$OUT_FILE" \
  cargo test --release --test tor_manager_metrics_tests "$TARGET_TEST"
popd >/dev/null

echo "[run_massif] profile written to $OUT_FILE"
