#!/usr/bin/env bash
# Build the Android release APK using Capacitor and the Rust backend.
#
# Usage:
#   ./mobile/scripts/build_android.sh
#
# Make sure `bun`, `cargo`, `npx`, `java` and the Android SDK (34+) are installed
# and that the ANDROID_HOME or ANDROID_SDK_ROOT environment variable is set.
set -euo pipefail

# Provide the line number of the failing command to ease troubleshooting
trap 'echo "[ERROR] Build aborted at line $LINENO. See output above for details." >&2' ERR

check_dep() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "[ERROR] '$1' is required but not installed." >&2
    missing=1
  fi
}

# Convenience helper for prominent error messages
error() {
  echo "[ERROR] $*" >&2
}


missing=0
REQUIRED_API=34

msg() {
  echo "[INFO] $*"
}

# Verify all required tools are present.
for cmd in bun cargo npx java gradle; do
  check_dep "$cmd"
done

if ! npx cap --version >/dev/null 2>&1; then
  error "Capacitor CLI not found. Run 'bun install' first."
  missing=1
fi

if [ -z "${ANDROID_HOME:-}" ] && [ -z "${ANDROID_SDK_ROOT:-}" ] && ! command -v sdkmanager >/dev/null 2>&1; then
  error "Android SDK not found. Set ANDROID_HOME or ANDROID_SDK_ROOT to the SDK path."
  missing=1
fi

SDK_PATH="${ANDROID_HOME:-$ANDROID_SDK_ROOT}"
if [ -n "$SDK_PATH" ] && [ ! -d "$SDK_PATH/platforms/android-$REQUIRED_API" ]; then
  error "Android SDK platform $REQUIRED_API missing in $SDK_PATH"
  echo "       Install with: sdkmanager \"platforms;android-$REQUIRED_API\"" >&2
  missing=1
fi

if [ "$missing" -eq 1 ]; then
  error "Missing dependencies detected. Aborting build."
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
msg "Synchronizing Capacitor project"
npx cap sync android
msg "Copying assets"
npx cap copy android
msg "Building Android project"
if ! npx cap build android; then
  error "Capacitor build failed. Ensure the Android SDK and Gradle are correctly installed."
  exit 1
fi

APK_PATH=$(find android/app/build/outputs/apk -name '*.apk' | head -n 1 || true)
if [ -n "$APK_PATH" ] && [ -f "$APK_PATH" ]; then
  DEST_DIR="$SCRIPT_DIR/../android"
  mkdir -p "$DEST_DIR"
  cp "$APK_PATH" "$DEST_DIR/"
  msg "APK copied to $DEST_DIR/$(basename "$APK_PATH")"
else
  error "No APK produced. Check the build log for details."
  exit 1
fi
