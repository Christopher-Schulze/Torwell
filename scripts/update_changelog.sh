#!/bin/bash
set -e

VERSION="$1"
CHANGELOG="docs/Changelog.md"

if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>" >&2
  exit 1
fi

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
