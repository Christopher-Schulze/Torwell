#!/usr/bin/env bash
set -e
SRC_DIR="$(dirname "$(dirname "$0")")/src/lib/components"
DEST_DIR="$(dirname "$(dirname "$0")")/src/lib/components_backup"
mkdir -p "$DEST_DIR"
rsync -a --delete "$SRC_DIR/" "$DEST_DIR/"
echo "UI components backed up to $DEST_DIR"
