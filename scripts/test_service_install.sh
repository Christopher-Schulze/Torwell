#!/usr/bin/env bash
set -e

TMP_DIR=$(mktemp -d)
echo "Testing service installation in temporary directory $TMP_DIR"
FAKE_SYSTEMCTL="$TMP_DIR/systemctl"

cat > "$FAKE_SYSTEMCTL" <<'SCRIPT'
#!/usr/bin/env bash
if [ "$1" = "--no-pager" ]; then
  shift
fi
echo "systemctl $@"
if [ "$1" = "status" ]; then
  echo "\u25cf torwell84.service - Fake Service"
  echo "   Loaded: loaded ($TARGET_DIR/torwell84.service; enabled)"
  echo "   Active: active (running)"
fi
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

