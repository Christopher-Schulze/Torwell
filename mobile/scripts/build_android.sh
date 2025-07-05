#!/usr/bin/env bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")/.."

# Build the Svelte frontend
bun run build

# Build the Android app using Capacitor
cd "$SCRIPT_DIR/.."
bun install
npx cap sync android
npx cap build android
