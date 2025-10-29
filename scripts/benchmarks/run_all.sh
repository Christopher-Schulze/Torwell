#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ARTIFACT_DIR="scripts/benchmarks/artifacts"
BASELINE_DIR="scripts/benchmarks/baselines"
mkdir -p "$ARTIFACT_DIR"

compare_with_baseline() {
  local current_value="$1"
  local baseline_value="$2"
  local max_regression_pct="$3"
  python - "$current_value" "$baseline_value" "$max_regression_pct" <<'PY'
import sys
current, baseline, pct = (float(x) for x in sys.argv[1:4])
allowed = baseline * (1 + pct / 100.0)
if current > allowed:
    raise SystemExit(f"Regression detected: current={current} baseline={baseline} allowed={allowed}")
PY
}

printf '\n[bench] Running Criterion suite\n'
pushd src-tauri >/dev/null
cargo bench --bench bootstrap -- --save-baseline current
popd >/dev/null

CRITERION_EST="src-tauri/target/criterion/bootstrap/new/estimates.json"
if [[ -f "$CRITERION_EST" ]]; then
  cp "$CRITERION_EST" "$ARTIFACT_DIR/criterion-estimates.json"
  current_mean=$(node -e "const fs=require('fs');const data=JSON.parse(fs.readFileSync('$CRITERION_EST','utf8'));console.log(data.mean.point_estimate);")
  baseline_cfg="$BASELINE_DIR/criterion-bootstrap.json"
  if [[ -f "$baseline_cfg" ]]; then
    baseline_mean=$(node -e "const fs=require('fs');const data=JSON.parse(fs.readFileSync('$baseline_cfg','utf8'));console.log(data.mean_ns);")
    max_reg=$(node -e "const fs=require('fs');const data=JSON.parse(fs.readFileSync('$baseline_cfg','utf8'));console.log(data.maxRegressionPct);")
    compare_with_baseline "$current_mean" "$baseline_mean" "$max_reg"
  fi
else
  echo "[bench] Warning: Criterion estimates not found at $CRITERION_EST" >&2
fi

printf '\n[bench] Running Playwright performance suite\n'
npm run bench:playwright

if [[ -f "$ARTIFACT_DIR/playwright-latest.json" ]]; then
  baseline="$BASELINE_DIR/playwright-baseline.json"
  node -e "
const fs = require('fs');
const latest = JSON.parse(fs.readFileSync('$ARTIFACT_DIR/playwright-latest.json', 'utf8'));
const baseline = JSON.parse(fs.readFileSync('$baseline', 'utf8'));
const allowed = (value) => value * (1 + baseline.maxRegressionPct / 100);
if (latest.metrics.firstContentfulPaintMs > allowed(baseline.fcp_ms)) {
  throw new Error('Playwright FCP regression');
}
if (latest.metrics.domContentLoadedMs > allowed(baseline.domContentLoaded_ms)) {
  throw new Error('Playwright DOMContentLoaded regression');
}
if (latest.metrics.loadEventMs > allowed(baseline.load_ms)) {
  throw new Error('Playwright load event regression');
}
" || exit 1
fi

printf '\n[bench] Benchmarks completed. Artifacts in %s\n' "$ARTIFACT_DIR"
