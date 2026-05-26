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
# Without Rust pre-baked there's no `cargo-binstall` either. Use cargo-binstall's
# own curl installer to pull a prebuilt — about a second, vs ~1m15s of `cargo
# install --locked` building it from source.
if ! command -v cargo-binstall >/dev/null 2>&1; then
	curl -L --proto '=https' --tlsv1.2 -sSf \
		https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh \
		| bash
fi
cargo binstall --no-confirm --force just
cargo binstall --no-confirm --force wasm-pack
# Skip cargo-binstall'ing `tauri-cli` — Tauri doesn't publish GitHub release
# prebuilts for the CLI, so binstall always falls back to a source compile
# (~2 minutes). `@tauri-apps/cli` from npm is the same Rust binary distributed
# via platform-specific subpackages — it's already a devDependency of
# `harper-desktop/package.json`, so `pnpm install` in that directory will pick
# it up. The justfile recipes invoke it as `pnpm tauri build` rather than
# `cargo tauri build`.

echo "--- :npm: Install Node + pnpm"
# Xcode CI agents don't bake in Node. Brew installs are idempotent (no-op if
# already present). Skipping corepack: Node 25+ no longer bundles it, and the
# `packageManager` pin in the root `package.json` is only a hint at this point
# — pnpm 10.x reads it but tolerates a minor version mismatch on its own.
if ! command -v node >/dev/null 2>&1; then
	brew install node
fi
if ! command -v pnpm >/dev/null 2>&1; then
	brew install pnpm
fi
hash -r

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
