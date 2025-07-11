#!/usr/bin/env bash
set -e

echo "Installing Torwell84 systemd service..."

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SERVICE_FILE="$ROOT_DIR/src-tauri/torwell84.service"
TARGET_DIR="${TARGET_DIR:-/etc/systemd/system}"
export TARGET_DIR
SYSTEMCTL="${SYSTEMCTL:-systemctl}"
SUDO="${SUDO-sudo}"

# Basic path validation
if [ ! -f "$SERVICE_FILE" ]; then
  echo "Service file not found: $SERVICE_FILE" >&2
  exit 1
fi

if [ ! -d "$TARGET_DIR" ]; then
  echo "Target directory $TARGET_DIR does not exist" >&2
  exit 1
fi

if ! command -v "$SYSTEMCTL" >/dev/null 2>&1; then
  echo "systemctl command not found: $SYSTEMCTL" >&2
  exit 1
fi

# Create service user and group if they do not exist
if ! id torwell >/dev/null 2>&1; then
  $SUDO useradd --system --user-group --home /opt/torwell84 torwell
fi

# Ensure application directory exists
$SUDO mkdir -p /opt/torwell84
if [ ! -f /opt/torwell84/torwell84 ]; then
  echo "Warning: /opt/torwell84/torwell84 not found" >&2
fi

# Copy service file to systemd directory
echo "Copying service file to $TARGET_DIR"
$SUDO cp "$SERVICE_FILE" "$TARGET_DIR/"
echo "Service file installed"

# Reload systemd manager configuration
$SUDO $SYSTEMCTL daemon-reload

# Enable and start the service immediately
echo "Enabling and starting torwell84.service"
$SUDO $SYSTEMCTL enable --now torwell84.service

echo "Service status:"
$SUDO $SYSTEMCTL --no-pager status torwell84.service
echo "Installation complete."

