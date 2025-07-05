#!/usr/bin/env bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")/.."


# Build the Svelte frontend
bun run build

# Compile the Rust backend with the `mobile` feature so the HTTP bridge is
# available when the app runs.
cargo build --release --manifest-path "$ROOT_DIR/src-tauri/Cargo.toml" --features mobile

# Build the iOS app using Capacitor
cd "$SCRIPT_DIR/.."
bun install
npx cap sync ios
npx cap build ios
