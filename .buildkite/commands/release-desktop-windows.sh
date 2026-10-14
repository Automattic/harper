#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."
VERSION=$(node -p "require('./harper-desktop/src-tauri/tauri.conf.json').version")
INSTALLER="harper-desktop/src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/Harper_${VERSION}_x64-setup.exe"

echo "--- :package: Download verified Windows installer"
# The Windows agent stores the artifact path with backslashes. Download patterns
# match the whole path, and the agent converts the separators when downloading.
buildkite-agent artifact download "*Harper_${VERSION}_x64-setup.exe" . --step build-desktop-windows
[[ -f "$INSTALLER" ]] || { echo "No Windows installer downloaded" >&2; exit 1; }

echo "--- :rocket: Upload Windows installer to the draft release"
node .buildkite/commands/upload-release.cjs "$BUILDKITE_TAG" "$INSTALLER"
