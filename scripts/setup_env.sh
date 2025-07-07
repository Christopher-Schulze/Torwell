#!/usr/bin/env bash
set -e

if [ "$(uname)" != "Linux" ]; then
  echo "setup_env.sh is intended for Linux systems only."
  exit 0
fi

sudo apt-get update
sudo apt-get install -y \
  libglib2.0-dev \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  pkg-config
