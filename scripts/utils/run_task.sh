#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

if [ $# -eq 0 ]; then
  echo "Usage: $(basename "$0") <task> [args...]" >&2
  exit 1
fi

if command -v task >/dev/null 2>&1; then
  exec task "$@"
elif command -v bunx >/dev/null 2>&1; then
  exec bunx --bun @go-task/cli "$@"
elif command -v npx >/dev/null 2>&1; then
  exec npx @go-task/cli "$@"
else
  echo "[run_task] No Task runner found. Install go-task or @go-task/cli." >&2
  exit 127
fi
