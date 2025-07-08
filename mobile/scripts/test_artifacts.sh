#!/usr/bin/env bash
set -euo pipefail

TYPE=${1:-all}
APK=$(ls mobile/android/*.apk 2>/dev/null || true)
IPA=$(ls mobile/ios/*.ipa 2>/dev/null || true)

status=0

if [ "$TYPE" != "ios" ]; then
  if [ -n "$APK" ]; then
    echo "Found APK: $APK"
    if unzip -t "$APK" >/dev/null; then
      if unzip -l "$APK" | grep -q "AndroidManifest.xml"; then
        echo "APK archive looks valid"
      else
        echo "AndroidManifest.xml missing in APK" >&2
        status=1
      fi
    else
      echo "APK archive appears corrupted" >&2
      status=1
    fi
  else
    echo "APK not found in mobile/android" >&2
    status=1
  fi
fi

if [ "$TYPE" != "android" ]; then
  if [ -n "$IPA" ]; then
    echo "Found IPA: $IPA"
    if unzip -t "$IPA" >/dev/null; then
      if unzip -l "$IPA" | grep -q "Payload/"; then
        echo "IPA archive looks valid"
      else
        echo "Payload folder missing in IPA" >&2
        status=1
      fi
    else
      echo "IPA archive appears corrupted" >&2
      status=1
    fi
  else
    echo "IPA not found in mobile/ios" >&2
    status=1
  fi
fi

exit $status
