#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

if ! command -v cargo-fuzz >/dev/null 2>&1; then
  cat <<'MSG'
[fuzz] Missing dependency: cargo-fuzz
Install with: cargo install cargo-fuzz
MSG
  exit 1
fi

FUZZ_TIME="${FUZZ_TIME:-45}"
TARGETS=(secure_http_headers bridge_presets)
CARGO_FUZZ_TOOLCHAIN="${CARGO_FUZZ_TOOLCHAIN:-nightly}"

if ! rustup toolchain list | grep -q "${CARGO_FUZZ_TOOLCHAIN}"; then
  cat <<MSG
[fuzz] Missing toolchain: ${CARGO_FUZZ_TOOLCHAIN}
Install with: rustup toolchain install ${CARGO_FUZZ_TOOLCHAIN}
MSG
  exit 1
fi

pushd src-tauri >/dev/null
for target in "${TARGETS[@]}"; do
  printf '\n[fuzz] Running %s (max_total_time=%ss)\n' "$target" "$FUZZ_TIME"
  cargo +"${CARGO_FUZZ_TOOLCHAIN}" fuzz run "$target" -- -max_total_time="$FUZZ_TIME"
done
popd >/dev/null

printf '\n[fuzz] Completed all fuzz targets.\n'
