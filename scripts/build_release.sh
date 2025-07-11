#!/usr/bin/env bash
# Build a release bundle of the Tauri application.
#
# Usage:
#   ./scripts/build_release.sh
#
# The script requires `bun` and `cargo` to be installed and writes the
# resulting bundles to `src-tauri/target/release/bundle`.
set -euo pipefail

echo "Building release bundles..."

check_dep() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Error: '$1' is required but not installed." >&2
    exit 1
  fi
}

for cmd in bun cargo; do
  check_dep "$cmd"
done

if [ -z "${TAURI_UPDATE_URL:-}" ]; then
  echo "Error: TAURI_UPDATE_URL is not set" >&2
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Install Node dependencies and build the Tauri project in release mode.
(cd "$ROOT_DIR" && bun install)
(cd "$ROOT_DIR" && bun run tauri build --features experimental-api)

echo "Bundles written to $ROOT_DIR/src-tauri/target/release/bundle"
