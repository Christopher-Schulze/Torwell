#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

log() {
  printf '[devcontainer] %s\n' "$*"
}

if command -v sudo >/dev/null 2>&1; then
  log "Updating apt repositories"
  sudo apt-get update -y
  log "Installing developer tooling packages"
  sudo apt-get install -y neovim ripgrep fd-find build-essential pkg-config libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
fi

log "Seeding Neovim defaults"
mkdir -p "$HOME/.config/nvim"
if [ ! -f "$HOME/.config/nvim/init.lua" ]; then
  cp "$ROOT/.devcontainer/nvim/init.lua" "$HOME/.config/nvim/init.lua"
fi

log "Running project bootstrap"
cd "$ROOT"
./scripts/utils/bootstrap.sh --non-interactive --skip-tests --source devcontainer || {
  log "Bootstrap failed" >&2
  exit 1
}
