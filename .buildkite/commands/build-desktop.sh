#!/usr/bin/env bash

set -euo pipefail

echo "--- :rubygems: Install gems"
install_gems

echo "--- :rust: Install Rust toolchain"
# A8C Mac VMs have no Rust tooling baked in. Install `rustup` non-interactively,
# then add the targets the Tauri universal build needs.
# `--no-modify-path` is intentional: we source `cargo/env` ourselves below so
# nothing outside this build step picks up cargo on a shared agent.
if ! command -v rustup >/dev/null 2>&1; then
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
		| sh -s -- -y --no-modify-path --default-toolchain stable --profile minimal
fi
# shellcheck disable=SC1091
. "$HOME/.cargo/env"
rustup target add aarch64-apple-darwin x86_64-apple-darwin wasm32-unknown-unknown

echo "--- :package: Install build tools"
# Without Rust pre-baked there's no `cargo-binstall` either. Bootstrap it via
# `cargo install` (~minute), then use it for the rest (downloads prebuilts).
if ! command -v cargo-binstall >/dev/null 2>&1; then
	cargo install cargo-binstall --locked
fi
cargo binstall --no-confirm --force just
cargo binstall --no-confirm --force tauri-cli
cargo binstall --no-confirm --force wasm-pack

echo "--- :npm: Enable corepack / install pnpm"
# Xcode CI agents don't bake in Node (or bake in too old a one without corepack).
# Install/upgrade via brew so we get a modern Node + corepack, then let corepack
# pick up the `packageManager` pin from the root `package.json` automatically.
if ! command -v corepack >/dev/null 2>&1; then
	brew install node
fi
corepack enable

echo "--- :key: Fetch Developer ID certificate"
bundle exec fastlane set_up_signing

echo "--- :hammer: Build harper-desktop (signed)"
# `just build-desktop-macos` runs `cargo tauri build -b app,dmg --target universal-apple-darwin`
# in `harper-desktop/`. With `APPLE_SIGNING_IDENTITY` set, Tauri's bundler signs the
# produced .app and .dmg. `TAURI_SIGNING_PRIVATE_KEY` enables the minisign updater signing.
export APPLE_SIGNING_IDENTITY="Developer ID Application: Automattic, Inc. (PZYM8XX95Q)"
just build-desktop-macos

echo "--- :apple: Notarize and staple"
# Tauri signs but does not notarize when only APPLE_SIGNING_IDENTITY is set.
# Notarize the .app and .dmg ourselves so Gatekeeper accepts them with no warning.
APP_BUNDLE=$(find target/universal-apple-darwin/release/bundle/macos -maxdepth 1 -name '*.app' -type d | head -1)
DMG_FILE=$(find target/universal-apple-darwin/release/bundle/dmg -maxdepth 1 -name '*.dmg' -type f | head -1)

[ -n "$APP_BUNDLE" ] || { echo "no .app produced"; exit 1; }
[ -n "$DMG_FILE" ]   || { echo "no .dmg produced"; exit 1; }

Tools/notarize.sh "$APP_BUNDLE"
Tools/notarize.sh "$DMG_FILE"
