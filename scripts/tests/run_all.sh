#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

log_step() {
  printf '\n[run_all] %s\n' "$1"
}

log_step "Synchronizing SvelteKit manifests"
npx --yes svelte-kit sync >/dev/null

log_step "Running frontend lint (ESLint)"
npm run lint

log_step "Running Rust lint (clippy)"
pushd src-tauri >/dev/null
cargo clippy --all-targets --all-features -- -D warnings
popd >/dev/null

log_step "Running frontend unit tests"
npm run test:unit

log_step "Running Rust unit tests"
pushd src-tauri >/dev/null
cargo test --lib --bins
popd >/dev/null

log_step "Running Rust integration tests"
pushd src-tauri >/dev/null
cargo test --tests
popd >/dev/null

log_step "Running UI snapshot tests"
npm run test:ui-snapshots

log_step "Verifying TLS compliance"
node scripts/tests/check_tls.js
