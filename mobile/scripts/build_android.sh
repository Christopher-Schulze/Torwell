#!/usr/bin/env bash
set -e
# Verify required commands are available
for cmd in bun cargo npx; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Error: $cmd is not installed or not in PATH" >&2
    exit 1
  fi
done
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")/.."


# Build the Svelte frontend
bun run build

# Compile the Rust backend with the `mobile` feature so the HTTP bridge is
# available when the app runs.
cargo build --release --manifest-path "$ROOT_DIR/src-tauri/Cargo.toml" --features mobile

# Build the Android app using Capacitor
cd "$SCRIPT_DIR/.."
bun install
npx cap sync android
npx cap build android
