#!/usr/bin/env bash
set -e

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Create service user and group if they do not exist
if ! id torwell >/dev/null 2>&1; then
  sudo useradd --system --user-group --home /opt/torwell84 torwell
fi

# Ensure application directory exists
sudo mkdir -p /opt/torwell84

# Copy service file to systemd directory
sudo cp "$ROOT_DIR/src-tauri/torwell84.service" /etc/systemd/system/

# Reload systemd manager configuration
sudo systemctl daemon-reload

# Enable and start the service immediately
sudo systemctl enable --now torwell84.service
