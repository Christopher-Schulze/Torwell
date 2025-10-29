#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT/src-tauri"

start=$(python3 - <<'PYTHON'
import time
print(time.perf_counter())
PYTHON
)

cargo check --all-targets >/tmp/torwell_rust_check.log 2>&1 || {
  cat /tmp/torwell_rust_check.log >&2
  exit 1
}

end=$(python3 - <<'PYTHON'
import time
print(time.perf_counter())
PYTHON
)

duration=$(python3 - <<PYTHON
start = float("$start")
end = float("$end")
print(f"{end - start:.2f}")
PYTHON
)

echo "[bench:rust] Cargo check completed in ${duration}s"
