#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

pushd "${PROJECT_ROOT}/src-tauri" >/dev/null
cargo bench --bench pixel_filters "$@"
popd >/dev/null
