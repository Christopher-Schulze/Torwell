#!/usr/bin/env bash
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

# Build the Svelte frontend only if the build directory doesn't exist
if [ -d "$ROOT_DIR/build" ]; then
  msg "Reusing existing frontend build at $ROOT_DIR/build"
else
  msg "Building frontend"
  (cd "$ROOT_DIR" && bun run build)
fi

# Compile the Rust backend with the `mobile` feature so the HTTP bridge is available when the app runs.
msg "Compiling Rust backend"
cargo build --release --manifest-path "$ROOT_DIR/src-tauri/Cargo.toml" --features mobile

# Build the Android app using Capacitor
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
