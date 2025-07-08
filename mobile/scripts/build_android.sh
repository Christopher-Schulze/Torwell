#!/usr/bin/env bash
# Build the Android release APK using Capacitor and the Rust backend.
#
# Usage:
#   ./mobile/scripts/build_android.sh
#
# Make sure `bun`, `cargo`, `npx`, `java` and the Android SDK are installed and
# that the ANDROID_HOME or ANDROID_SDK_ROOT environment variable is set.
set -euo pipefail

trap 'echo "[ERROR] Build failed at line $LINENO" >&2' ERR

check_dep() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "[ERROR] '$1' is required but not installed." >&2
    missing=1
  fi
}


missing=0

msg() {
  echo "[INFO] $*"
}

# Verify all required tools are present.
for cmd in bun cargo npx java gradle; do
  check_dep "$cmd"
done

if ! npx cap --version >/dev/null 2>&1; then
  echo "[ERROR] Capacitor CLI not found. Run 'bun install' first." >&2
  missing=1
fi

if [ -z "${ANDROID_HOME:-}" ] && [ -z "${ANDROID_SDK_ROOT:-}" ] && ! command -v sdkmanager >/dev/null 2>&1; then
  echo "[ERROR] Android SDK not found. Please set ANDROID_HOME or ANDROID_SDK_ROOT." >&2
  missing=1
fi

if [ "$missing" -eq 1 ]; then
  echo "[ERROR] Missing dependencies detected. Aborting." >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Build the Svelte frontend if no previous build is available.
if [ -d "$ROOT_DIR/build" ]; then
  msg "Reusing existing frontend build at $ROOT_DIR/build"
else
  msg "Building frontend"
  (cd "$ROOT_DIR" && bun run build)
fi

# Compile the Rust backend with mobile support so the HTTP bridge is available.
msg "Compiling Rust backend"
cargo build --release --manifest-path "$ROOT_DIR/src-tauri/Cargo.toml" --features mobile

# Build the Android app using Capacitor.
cd "$SCRIPT_DIR/.."
bun install
msg "Copying assets"
npx cap copy android
msg "Building Android project"
npx cap build android

APK_PATH=$(find android/app/build/outputs/apk -name '*.apk' | head -n 1 || true)
if [ -n "$APK_PATH" ] && [ -f "$APK_PATH" ]; then
  DEST_DIR="$SCRIPT_DIR/../android"
  mkdir -p "$DEST_DIR"
  cp "$APK_PATH" "$DEST_DIR/"
  msg "APK copied to $DEST_DIR/$(basename "$APK_PATH")"
else
  echo "[ERROR] No APK produced" >&2
  exit 1
fi
