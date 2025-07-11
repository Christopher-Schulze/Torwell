#!/usr/bin/env bash
# Build the iOS release IPA using Capacitor and the Rust backend.
#
# Usage:
#   ./mobile/scripts/build_ios.sh
#
# Requires `bun`, `cargo`, `npx`, Xcode tools and CocoaPods installed.
# iOS SDK 17 or newer is recommended.
set -euo pipefail

# Provide the failing line to make troubleshooting easier
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
REQUIRED_IOS_SDK=17

msg() {
  echo "[INFO] $*"
}

# Verify required build tools.
for cmd in bun cargo npx xcodebuild pod; do
  check_dep "$cmd"
done

if ! npx cap --version >/dev/null 2>&1; then
  error "Capacitor CLI not found. Run 'bun install' first."
  missing=1
fi

IOS_SDK_VERSION=$(xcrun --sdk iphoneos --show-sdk-version 2>/dev/null || echo 0)
IOS_MAJOR=${IOS_SDK_VERSION%%.*}
if [ "$IOS_MAJOR" -lt "$REQUIRED_IOS_SDK" ]; then
  error "iOS SDK $REQUIRED_IOS_SDK or newer required (found $IOS_SDK_VERSION)"
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

# Build the iOS app using Capacitor.
cd "$SCRIPT_DIR/.."
bun install
msg "Synchronizing Capacitor project"
npx cap sync ios
msg "Copying assets"
npx cap copy ios
msg "Building iOS project"
if ! npx cap build ios; then
  error "Capacitor build failed. Ensure Xcode and CocoaPods are correctly installed."
  exit 1
fi

IPA_PATH=$(find ios -name '*.ipa' | head -n 1 || true)
if [ -n "$IPA_PATH" ] && [ -f "$IPA_PATH" ]; then
  DEST_DIR="$SCRIPT_DIR/../ios"
  mkdir -p "$DEST_DIR"
  cp "$IPA_PATH" "$DEST_DIR/"
  msg "IPA copied to $DEST_DIR/$(basename "$IPA_PATH")"
else
  error "No IPA produced. Check the build log for details."
  exit 1
fi
