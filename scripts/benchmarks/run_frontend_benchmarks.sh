#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

if ! command -v bun >/dev/null 2>&1; then
  echo "[benchmarks] Fehler: 'bun' wurde nicht gefunden. Installiere Bun >=1.1." >&2
  exit 1
fi

pushd "${PROJECT_ROOT}" >/dev/null
trap 'popd >/dev/null' EXIT

echo "[benchmarks] Starte Vitest Benchmarks"
# `bun x` ruft vitest aus den lokalen Dependencies auf
bun x vitest bench "$@"

echo "[benchmarks] Benchmarks abgeschlossen"
