#!/usr/bin/env bash
# Builds build/HarperMac Safari.app: a small host app that carries the Safari extension.
#
# Safari extensions must live inside a Mac app. Apple's `safari-web-extension-converter` wraps
# the web extension from ../harper-mac-chrome in one, and xcodebuild compiles it.
# Needs full Xcode (not just the Command Line Tools); xcode-select need not point at it.
set -euo pipefail
cd "$(dirname "$0")"

XCODE="${XCODE:-/Applications/Xcode.app}"
[ -d "$XCODE" ] || { echo "Xcode not found at $XCODE (set XCODE=...)" >&2; exit 1; }
export DEVELOPER_DIR="$XCODE/Contents/Developer"

WORK=build/work
rm -rf "$WORK"
mkdir -p "$WORK/extension"

# Copy only the extension itself, so the README and icon script don't end up inside the bundle.
SRC=../harper-mac-chrome
cp -R "$SRC/manifest.json" "$SRC/background.js" "$SRC/content.js" \
      "$SRC/popup.html" "$SRC/popup.js" "$SRC/icons" "$WORK/extension/"

xcrun safari-web-extension-converter "$WORK/extension" \
  --project-location "$WORK" --app-name "HarperMac Safari" \
  --bundle-identifier com.harryrose.harpermac.safari \
  --macos-only --no-open --no-prompt --copy-resources

# The converter gives the host app an id that isn't a prefix of the extension's; Xcode rejects that.
PBX="$WORK/HarperMac Safari/HarperMac Safari.xcodeproj/project.pbxproj"
sed -i '' 's/com.harryrose.harpermac.HarperMac-Safari/com.harryrose.harpermac.safari/' "$PBX"

# Signed ad hoc: fine for local use, but Safari then needs Develop > Allow Unsigned Extensions.
xcodebuild -project "$WORK/HarperMac Safari/HarperMac Safari.xcodeproj" \
  -scheme "HarperMac Safari" -configuration Release -derivedDataPath "$WORK/DerivedData" \
  CODE_SIGN_IDENTITY="-" CODE_SIGNING_REQUIRED=NO CODE_SIGNING_ALLOWED=YES DEVELOPMENT_TEAM="" \
  build | grep -E "error:|BUILD" || true

rm -rf "build/HarperMac Safari.app"
cp -R "$WORK/DerivedData/Build/Products/Release/HarperMac Safari.app" build/
echo "Built build/HarperMac Safari.app"
