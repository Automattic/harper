#!/usr/bin/env bash
# Buildkite step: build, sign, and notarize harper-desktop.
# See .buildkite/pipeline.yml.

set -euo pipefail

echo "--- :rubygems: Install gems"
install_gems

echo "--- :rust: Install Rust toolchain"
# A8C Mac VMs have no Rust tooling baked in. Install `rustup` non-interactively,
# then add the targets we need.
# `--no-modify-path` is intentional: we source `cargo/env` ourselves below so
# nothing outside this build step picks up cargo on a shared agent.
if ! command -v rustup >/dev/null 2>&1; then
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
		| sh -s -- -y --no-modify-path --default-toolchain stable --profile minimal
fi
# shellcheck disable=SC1091
. "$HOME/.cargo/env"

# Universal builds (master, tags) need both arches. PR/branch builds run
# Apple-Silicon-only to skip the x86_64 half of the compile (~4m saved).
if [ "${BUILDKITE_BRANCH:-}" = "master" ] || [ -n "${BUILDKITE_TAG:-}" ]; then
	TAURI_TARGET=universal-apple-darwin
	JUST_RECIPE=build-desktop-macos
	rustup target add aarch64-apple-darwin x86_64-apple-darwin wasm32-unknown-unknown
else
	TAURI_TARGET=aarch64-apple-darwin
	JUST_RECIPE=build-desktop-macos-arm64
	rustup target add aarch64-apple-darwin wasm32-unknown-unknown
fi
echo "Build target: $TAURI_TARGET (recipe: $JUST_RECIPE)"

echo "--- :floppy_disk: Restore cargo caches"
# Two cache layers, both keyed off Cargo.lock + rust-toolchain.toml so they
# invalidate the moment any dep version moves.
#   1. `~/.cargo/registry` + `~/.cargo/git/db` — downloaded crate sources / git
#      deps. ~340MB compressed. Saves the crate-download phase on cold builds.
#   2. `target/` — compiled artifacts. Several GB compressed. Lets cargo's
#      incremental compilation reuse object files across builds. The target
#      cache also includes the Tauri target arch in its key, since universal
#      and arm64-only builds produce disjoint object trees.
CARGO_DEPS_KEY="$BUILDKITE_PIPELINE_SLUG-cargo-deps-darwin-arm64-$(hash_file Cargo.lock)-$(hash_file rust-toolchain.toml)"
CARGO_TARGET_KEY="$BUILDKITE_PIPELINE_SLUG-target-darwin-arm64-$TAURI_TARGET-$(hash_file Cargo.lock)-$(hash_file rust-toolchain.toml)"
restore_cache "$CARGO_DEPS_KEY"
restore_cache "$CARGO_TARGET_KEY"

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

echo "--- :npm: Install pnpm"
# Node is set up by the `automattic/nvm` Buildkite plugin from `.nvmrc`.
# `pnpm` is not bundled — install it via `npm install -g`. Pin matches the
# `packageManager` field in the root `package.json`.
npm install -g pnpm@10.10.0
hash -r

echo "--- :key: Fetch Developer ID certificate"
bundle exec fastlane set_up_signing

echo "--- :hammer: Build harper-desktop (signed)"
# `just $JUST_RECIPE` runs `pnpm tauri build -b app,dmg --target $TAURI_TARGET`
# in `harper-desktop/`. With `APPLE_SIGNING_IDENTITY` set, Tauri's bundler signs
# the produced .app and .dmg. `TAURI_SIGNING_PRIVATE_KEY` enables the minisign
# updater signing.
export APPLE_SIGNING_IDENTITY="Developer ID Application: Automattic, Inc. (PZYM8XX95Q)"
just "$JUST_RECIPE"

echo "--- :floppy_disk: Save cargo caches"
# `save_cache` is a no-op when the key already exists in S3, so this is cheap on
# subsequent same-deps builds. Runs after the build (so what gets cached is
# coherent) but before notarization (a flaky notary shouldn't lose us the
# cache). The target/ save is the one to watch — several GB of upload on a cold
# key.
save_cache "$HOME/.cargo/registry" "$CARGO_DEPS_KEY"
save_cache "$HOME/.cargo/git/db" "$CARGO_DEPS_KEY-git" || true
save_cache target "$CARGO_TARGET_KEY"

echo "--- :apple: Notarize and staple"
# Tauri signs but does not notarize when only APPLE_SIGNING_IDENTITY is set.
# Notarize the .app and .dmg ourselves so Gatekeeper accepts them with no warning.
APP_BUNDLE=$(find "target/$TAURI_TARGET/release/bundle/macos" -maxdepth 1 -name '*.app' -type d | head -1)
DMG_FILE=$(find "target/$TAURI_TARGET/release/bundle/dmg" -maxdepth 1 -name '*.dmg' -type f | head -1)

[ -n "$APP_BUNDLE" ] || { echo "no .app produced"; exit 1; }
[ -n "$DMG_FILE" ]   || { echo "no .dmg produced"; exit 1; }

Tools/notarize.sh "$APP_BUNDLE"
Tools/notarize.sh "$DMG_FILE"
