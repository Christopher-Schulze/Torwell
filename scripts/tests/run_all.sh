#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
FRONTEND_DIR="${PROJECT_ROOT}"
BACKEND_DIR="${PROJECT_ROOT}/src-tauri"

log() {
  printf "[run_all] %s\n" "$1"
}

if ! command -v bun >/dev/null 2>&1; then
  log "Fehler: 'bun' wurde nicht gefunden. Bitte Bun >=1.1 installieren."
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  log "Fehler: 'cargo' wurde nicht gefunden. Bitte Rust/Cargo >=1.77 installieren."
  exit 1
fi

EXTRA_ARGS=("$@")

pushd "${FRONTEND_DIR}" >/dev/null
log "Starte 'bun run check'"
bun run check

log "Starte 'bun run test'"
bun run test
popd >/dev/null

pushd "${BACKEND_DIR}" >/dev/null
log "Starte 'cargo test --locked'"
cargo test --locked "${EXTRA_ARGS[@]}"
popd >/dev/null

log "Alle Tests erfolgreich abgeschlossen."
