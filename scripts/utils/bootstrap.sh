#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
NON_INTERACTIVE=false
SKIP_TESTS=false
SOURCE="manual"

usage() {
  cat <<USAGE
Usage: $(basename "$0") [options]
  --non-interactive  Do not prompt, fail on missing prerequisites
  --skip-tests       Skip verification test suite
  --skip-hooks       Do not configure git hooks
  --source <name>    Tag output with invoker (e.g. devcontainer)
USAGE
}

CONFIGURE_HOOKS=true

while [ $# -gt 0 ]; do
  case "$1" in
    --non-interactive)
      NON_INTERACTIVE=true
      ;;
    --skip-tests)
      SKIP_TESTS=true
      ;;
    --skip-hooks)
      CONFIGURE_HOOKS=false
      ;;
    --source)
      SOURCE="$2"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
  shift
done

log() {
  printf '[bootstrap:%s] %s\n' "$SOURCE" "$*"
}

require_bin() {
  local bin="$1"
  if ! command -v "$bin" >/dev/null 2>&1; then
    log "Missing dependency: $bin"
    if [ "$NON_INTERACTIVE" = true ]; then
      exit 2
    fi
    read -rp "Install $bin now? [y/N] " reply || exit 3
    if [[ ! "$reply" =~ ^[Yy]$ ]]; then
      log "Aborting due to missing $bin"
      exit 3
    fi
  fi
}

log "Ensuring required toolchain is available"
for bin in bun cargo rustup; do
  require_bin "$bin"
done

if ! command -v task >/dev/null 2>&1; then
  log "go-task not found, ensuring @go-task/cli via bun"
  require_bin bunx
fi

log "Installing JavaScript dependencies via bun"
cd "$ROOT"
bun install --frozen-lockfile

log "Syncing Svelte kit configuration"
bun run check -- --output summary >/dev/null 2>&1 || true

log "Fetching Rust dependencies"
if command -v cargo >/dev/null 2>&1; then
  (cd "$ROOT/src-tauri" && cargo fetch)
fi

if [ "$CONFIGURE_HOOKS" = true ]; then
  log "Configuring git hooks path"
  git config core.hooksPath .githooks
  chmod +x .githooks/* 2>/dev/null || true
fi

if [ ! -f "$ROOT/.env" ] && [ -f "$ROOT/.env.example" ]; then
  log "Creating default .env from example"
  cp "$ROOT/.env.example" "$ROOT/.env"
fi

if [ "$SKIP_TESTS" = false ]; then
  log "Running verification tasks"
  ./scripts/utils/run_task.sh lint
  ./scripts/utils/run_task.sh test:quick
else
  log "Skipping verification tasks"
fi

log "Bootstrap completed"
