#!/usr/bin/env bash
set -euo pipefail

# Benchmark Tor bootstrap latency by invoking the production build via Tauri CLI.
# Requires the Torwell desktop app dependencies and Tor network access.
# Outputs p50/p95/p99 bootstrap durations and logs raw samples for inspection.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RESULTS_DIR="${ROOT_DIR}/.benchmarks"
SAMPLES="${RESULTS_DIR}/bootstrap_samples.csv"
SUMMARY="${RESULTS_DIR}/bootstrap_summary.txt"
ITERATIONS=${ITERATIONS:-10}
TIMEOUT=${TIMEOUT:-180}

mkdir -p "${RESULTS_DIR}"
: > "${SAMPLES}"

log() {
  printf '[%s] %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$*"
}

measure_bootstrap() {
  local iteration="$1"
  log "Iteration ${iteration}/${ITERATIONS}: starting Tor bootstrap benchmark"
  local start_ts end_ts duration
  start_ts=$(date +%s%3N)

  # Launch the Tauri CLI bootstrap command. This assumes `task desktop:bootstrap`
  # performs a connection attempt and exits once the bootstrap is complete.
  if ! timeout "${TIMEOUT}" task desktop:bootstrap >/tmp/torwell_bootstrap.log 2>&1; then
    log "Iteration ${iteration}: bootstrap command timed out or failed"
    return 1
  fi

  end_ts=$(date +%s%3N)
  duration=$((end_ts - start_ts))
  log "Iteration ${iteration}: bootstrap completed in ${duration} ms"
  printf '%s,%s\n' "${iteration}" "${duration}" >>"${SAMPLES}"
}

main() {
  log "Writing benchmark samples to ${SAMPLES}"
  for i in $(seq 1 "${ITERATIONS}"); do
    if ! measure_bootstrap "${i}"; then
      log "Benchmark aborted at iteration ${i}" >&2
      exit 1
    fi
  done

  log "Computing summary statistics"
  python3 - <<'PY'
import csv
import math
import pathlib

samples = []
root = pathlib.Path("${SAMPLES}")
with root.open() as fh:
    reader = csv.reader(fh)
    for row in reader:
        try:
            samples.append(int(row[1]))
        except (ValueError, IndexError):
            continue

if not samples:
    raise SystemExit("no samples recorded")

samples.sort()

def percentile(values, pct):
    if not values:
        return math.nan
    k = (len(values) - 1) * (pct / 100)
    f = math.floor(k)
    c = math.ceil(k)
    if f == c:
        return values[int(k)]
    d0 = values[f] * (c - k)
    d1 = values[c] * (k - f)
    return d0 + d1

summary = {
    "iterations": len(samples),
    "p50_ms": percentile(samples, 50),
    "p95_ms": percentile(samples, 95),
    "p99_ms": percentile(samples, 99),
    "max_ms": max(samples),
    "min_ms": min(samples),
}

summary_path = pathlib.Path("${SUMMARY}")
with summary_path.open("w", encoding="utf8") as out:
    for key, value in summary.items():
        out.write(f"{key}={value}\n")

print(f"Summary written to {summary_path}")
PY
}

main "$@"
