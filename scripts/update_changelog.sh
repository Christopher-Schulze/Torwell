#!/bin/bash
set -e

CHANGELOG="docs/Changelog.md"

# Read version from package.json and Cargo.toml and ensure they match
PKG_VERSION=$(jq -r .version package.json)
CARGO_VERSION=$(grep -m1 '^version' src-tauri/Cargo.toml | cut -d '"' -f2)

if [ "$PKG_VERSION" != "$CARGO_VERSION" ]; then
  echo "Version mismatch between package.json and Cargo.toml" >&2
  exit 1
fi

VERSION="$PKG_VERSION"

DATE=$(date +%Y-%m-%d)
PREVIOUS_TAG=$(git tag --sort=-v:refname | sed -n '2p')

{
  echo "## [$VERSION] - $DATE"
  if [ -n "$PREVIOUS_TAG" ]; then
    git log "$PREVIOUS_TAG"..HEAD --pretty=format:"- %s" --no-merges
  else
    git log --pretty=format:"- %s" --no-merges
  fi
  echo
} > .changelog.tmp

cat "$CHANGELOG" >> .changelog.tmp
mv .changelog.tmp "$CHANGELOG"

# Optionally trigger a certificate update on release
if [ -n "$CERT_UPDATE_URL" ]; then
  curl -fsS -X POST "$CERT_UPDATE_URL" || \
    echo "Warning: certificate update request failed" >&2
fi
