#!/usr/bin/env bash
# Builds build/HarperMac.app (release) with harper-ls bundled inside.
set -euo pipefail
cd "$(dirname "$0")"

[ -x vendor/harper-ls/harper-ls ] || { echo "run tools/fetch_harper.sh first" >&2; exit 1; }

swift build -c release
BIN="$(swift build -c release --show-bin-path)/HarperMac"

APP=build/HarperMac.app
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/HarperMac"
cp Info.plist "$APP/Contents/Info.plist"
cp vendor/harper-ls/harper-ls "$APP/Contents/Resources/harper-ls"

# Ad-hoc signing is enough to run locally. Note macOS ties the Accessibility grant to the
# signature, so after a rebuild you may need to remove and re-add the app in
# System Settings > Privacy & Security > Accessibility.
codesign --force --deep --sign - "$APP"
echo "Built $APP"
