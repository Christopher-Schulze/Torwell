#!/usr/bin/env bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")/.."

# Build the Svelte frontend
bun run build

# Build the iOS app using Capacitor
cd "$SCRIPT_DIR/.."
bun install
npx cap sync ios
npx cap build ios
