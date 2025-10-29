#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/scripts/tests/artifacts"
CACHE_DIR="$ARTIFACT_DIR/cache"
OUTPUT_PNG="$ARTIFACT_DIR/renderer_frame.png"

mkdir -p "$CACHE_DIR"
rm -f "$OUTPUT_PNG"

echo "[headless_renderer] capturing headless frame"
if TORWELL_SHADER_CACHE_DIR="$CACHE_DIR" \
    cargo run --manifest-path "$ROOT/src-tauri/Cargo.toml" --bin renderer_capture -- --output "$OUTPUT_PNG"; then
    if [[ -f "$OUTPUT_PNG" ]]; then
        echo "[headless_renderer] output saved to $OUTPUT_PNG"
    else
        echo "[headless_renderer] renderer skipped (no output)"
    fi
else
    echo "[headless_renderer] capture failed" >&2
    exit 1
fi
