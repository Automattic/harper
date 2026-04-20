clean:
  rm -rf harper

# Pull and checkout the source code for Harper's monorepo. Necessary for using unreleased features or packages.
pull-dep-source:
  #! /bin/bash
  set -eo pipefail

  just clean

  git clone https://github.com/automattic/harper
  cd harper
  git switch editor-package

# Build the necessary dependenceies from the Harper monorepo
build-harper-deps:
  #! /bin/bash

  if [ ! -d "harper" ]; then
    just pull-dep-source
  fi

  cd harper && just build-harperjs build-harper-editor

dev:
  cargo tauri dev

prebuild: build-harper-deps
  cargo tauri info
  pnpm install

build-linux: prebuild
  cargo tauri build -b deb,rpm

build-macos: prebuild
  cargo tauri build -b app,dmg

build-windows: prebuild
  cargo tauri build -b msi
