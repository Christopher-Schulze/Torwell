#!/usr/bin/env bash
set -e

check_dep() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Error: '$1' is required but not installed." >&2
    exit 1
  fi
}

for cmd in bun cargo; do
  check_dep "$cmd"
done

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

(cd "$ROOT_DIR" && bun install)
(cd "$ROOT_DIR" && bun run tauri build)

echo "Bundles written to $ROOT_DIR/src-tauri/target/release/bundle"
