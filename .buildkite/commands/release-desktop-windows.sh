#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."
VERSION=$(jq -r '.version' harper-desktop/src-tauri/tauri.conf.json)
[[ "${BUILDKITE_TAG:-}" = "v$VERSION" ]] || { echo "Release tag must match v$VERSION" >&2; exit 1; }
INSTALLER="harper-desktop/src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/Harper_${VERSION}_x64-setup.exe"

echo "--- :package: Download verified Windows installer"
# Select the producing step in this build, never an artifact from another run.
buildkite-agent artifact download "$INSTALLER" . --step build-desktop-windows
[[ -f "$INSTALLER" ]] || { echo "No Windows installer downloaded" >&2; exit 1; }

echo "--- :rubygems: Install gems"
install_gems

echo "--- :rocket: Upload Windows installer to the draft release"
bundle exec fastlane upload_windows_github_release tag:"$BUILDKITE_TAG" installer:"$INSTALLER"
