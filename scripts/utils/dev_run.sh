#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

if ! command -v bun >/dev/null 2>&1; then
  echo "[dev_run] Fehler: 'bun' wurde nicht gefunden. Installiere Bun >=1.1." >&2
  exit 1
fi

pushd "${PROJECT_ROOT}" >/dev/null
trap 'popd >/dev/null' EXIT

export RUST_LOG=${RUST_LOG:-info}
export TAURI_DEV_WATCHER_IGNORE=${TAURI_DEV_WATCHER_IGNORE:-"logs/**"}

echo "[dev_run] Starte Tauri Dev-Session (bun run tauri:dev)"
exec bun run tauri:dev "$@"
