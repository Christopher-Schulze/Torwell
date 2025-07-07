#!/usr/bin/env bash
set -e

TMP_DIR=$(mktemp -d)
FAKE_SYSTEMCTL="$TMP_DIR/systemctl"

# create a fake systemctl that just echoes the arguments
cat > "$FAKE_SYSTEMCTL" <<'SCRIPT'
#!/usr/bin/env bash
echo "systemctl $@"
SCRIPT
chmod +x "$FAKE_SYSTEMCTL"

export TARGET_DIR="$TMP_DIR"
export SYSTEMCTL="$FAKE_SYSTEMCTL"
export SUDO=""

./scripts/install_service.sh

if [ -f "$TMP_DIR/torwell84.service" ]; then
  echo "Service file installed in $TMP_DIR"
else
  echo "Service installation failed" >&2
  exit 1
fi

echo "Test completed successfully"
