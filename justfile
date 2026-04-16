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
  cd harper && just build-harper-editor
