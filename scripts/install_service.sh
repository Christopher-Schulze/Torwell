#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
SERVICE_FILE="$REPO_ROOT/src-tauri/torwell84.service"

sudo cp "$SERVICE_FILE" /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now torwell84.service
