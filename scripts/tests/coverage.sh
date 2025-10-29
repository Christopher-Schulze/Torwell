#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

mkdir -p coverage/frontend coverage/rust

if ! command -v cargo-tarpaulin >/dev/null 2>&1; then
  cat <<'MSG'
[cov] Missing dependency: cargo-tarpaulin
Install with: cargo install cargo-tarpaulin
MSG
  exit 1
fi

printf '\n[cov] Running frontend coverage (Vitest)\n'
npx --yes vitest run --coverage

printf '\n[cov] Running Rust coverage (cargo tarpaulin)\n'
pushd src-tauri >/dev/null
cargo tarpaulin --workspace --timeout 300 --out Html --out Xml --output-dir "${ROOT}/coverage/rust"
popd >/dev/null

printf '\n[cov] Coverage artifacts written to %s/coverage\n' "$ROOT"
