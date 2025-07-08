#!/usr/bin/env bash
set -euo pipefail

TYPE=${1:-all}
APK=$(ls mobile/android/*.apk 2>/dev/null || true)
IPA=$(ls mobile/ios/*.ipa 2>/dev/null || true)

status=0

if [ "$TYPE" != "ios" ]; then
  if [ -n "$APK" ]; then
    echo "Found APK: $APK"
  else
    echo "APK not found in mobile/android" >&2
    status=1
  fi
fi

if [ "$TYPE" != "android" ]; then
  if [ -n "$IPA" ]; then
    echo "Found IPA: $IPA"
  else
    echo "IPA not found in mobile/ios" >&2
    status=1
  fi
fi

exit $status
