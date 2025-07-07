#!/usr/bin/env bash
set -e

# Copy service file to systemd directory
sudo cp "$(dirname "$(dirname "$0")")/src-tauri/torwell84.service" /etc/systemd/system/

# Reload systemd manager configuration
sudo systemctl daemon-reload

# Enable and start the service immediately
sudo systemctl enable --now torwell84.service
