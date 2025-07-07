#!/usr/bin/env bash
set -e

check_dep() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Error: '$1' is required but not installed." >&2
    exit 1
  fi
}

for cmd in bun cargo npx; do
  check_dep "$cmd"
done
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")/.."


# Build the Svelte frontend only if the build directory doesn't exist
if [ -d "$ROOT_DIR/build" ]; then
  echo "Reusing existing frontend build at $ROOT_DIR/build"
else
  (cd "$ROOT_DIR" && bun run build)
fi

# Compile the Rust backend with the `mobile` feature so the HTTP bridge is
# available when the app runs.
cargo build --release --manifest-path "$ROOT_DIR/src-tauri/Cargo.toml" --features mobile

# Build the Android app using Capacitor
cd "$SCRIPT_DIR/.."
bun install
npx cap copy android
npx cap build android
