# Format entire project
format:
  cargo fmt  
  cd "{{justfile_directory()}}/packages"; yarn prettier -w .

# Build the WebAssembly for a specific target (usually either `web` or `bundler`)
build-wasm target:
  cd "{{justfile_directory()}}/harper-wasm" && wasm-pack build --target {{target}}

# Build `harper.js` with all size optimizations available.
build-harperjs:
  #! /bin/bash
  set -eo pipefail
  just build-wasm web

  # Removes a duplicate copy of the WASM binary if Vite is left to its devices.
  sed -i 's/new URL(.*)/new URL()/g' "{{justfile_directory()}}/harper-wasm/pkg/harper_wasm.js"

  cd "{{justfile_directory()}}/packages/harper.js"
  yarn install -f
  yarn run build

  # Generate API reference
  ./docs.sh

test-harperjs:
  #!/bin/bash
  set -eo pipefail
  just build-harperjs
  
  cd "{{justfile_directory()}}/packages/harper.js"
  yarn install -f
  yarn playwright install
  yarn test

  # Test runnable examples
  cd "{{justfile_directory()}}/packages/harper.js/examples/commonjs-simple"
  yarn install
  yarn start

# Compile the website's dependencies and start a development server. Note that if you make changes to `harper-wasm`, you will have to re-run this command.
dev-web:
  #! /bin/bash
  set -eo pipefail

  just build-harperjs

  cd "{{justfile_directory()}}/packages/web"
  yarn install -f
  yarn dev

# Build the Harper website.
build-web:
  #! /bin/bash
  set -eo pipefail
  
  just build-harperjs
  
  cd "{{justfile_directory()}}/packages/web"
  yarn install -f
  yarn run build

# Build the Harper Obsidian plugin.
build-obsidian:
  #! /bin/bash
  set -eo pipefail
  
  just build-harperjs
  cd "{{justfile_directory()}}/packages/obsidian-plugin"

  yarn install -f
  yarn build

  zip harper-obsidian-plugin.zip manifest.json main.js

# Run VSCode plugin unit and integration tests.
test-vscode:
  #! /bin/bash
  set -eo pipefail

  ext_dir="{{justfile_directory()}}/packages/vscode-plugin"
  bin_dir="${ext_dir}/bin"

  if ! [[ -d "$bin_dir" ]]; then
    mkdir "$bin_dir"
  fi

  cargo build --release
  cp "{{justfile_directory()}}/target/release/harper-ls"* "$bin_dir"

  cd "$ext_dir"

  yarn install -f
  # For environments without displays like CI servers or containers
  if [[ "$(uname)" == "Linux" ]] && [[ -z "$DISPLAY" ]]; then
    xvfb-run --auto-servernum yarn test
  else
    yarn test
  fi

# Build and package the Visual Studio Code extension.
# If `target` is passed, it is assumed that `harper-ls` has been compiled beforehand and is in `packages/vscode-plugin/bin`. This is used in CI.
package-vscode target="":
  #! /bin/bash
  set -eo pipefail

  ext_dir="{{justfile_directory()}}/packages/vscode-plugin"
  bin_dir="${ext_dir}/bin"

  cp LICENSE "$ext_dir"

  if [[ -z "{{target}}" ]]; then
    cargo build --release

    if ! [[ -d "$bin_dir" ]]; then
      mkdir "$bin_dir"
    fi

    cp "{{justfile_directory()}}/target/release/harper-ls"* "$bin_dir"
  fi

  cd "$ext_dir"

  yarn install -f
  if [[ -n "{{target}}" ]]; then
    yarn package --target {{target}}
  else
    yarn package
  fi

update-vscode-linters:
  #! /bin/bash
  set -eo pipefail

  linters=$(
    cargo run --bin harper-cli -- config |
      jq 'with_entries(.key |= "harper-ls.linters." + . |
        .value |= {
          "scope": "resource",
          "type": "boolean",
          "default": .default_value,
          "description": .description
        }
      )'
  )

  cd "{{justfile_directory()}}/packages/vscode-plugin"

  manifest_without_linters=$(
    jq 'walk(
      if type == "object" then
        with_entries(select(.key | startswith("harper-ls.linters") | not))
      end
    )' package.json
  )

  jq --argjson linters "$linters" \
    '.contributes.configuration.properties += $linters' <<< \
    "$manifest_without_linters" > \
    package.json
  yarn prettier --write package.json

# Run Rust formatting and linting.
check-rust:
  #! /bin/bash
  set -eo pipefail

  cargo fmt -- --check
  cargo clippy -- -Dwarnings -D clippy::dbg_macro -D clippy::needless_raw_string_hashes

# Perform format and type checking.
check:
  #! /bin/bash
  set -eo pipefail

  just check-rust

  cd "{{justfile_directory()}}/packages"
  yarn install
  yarn prettier --check .
  yarn eslint .

  # Needed because Svelte has special linters
  cd web
  just build-web
  yarn run check

# Populate build caches and install necessary local tooling (tools callable via `yarn run <tool>`).
setup:
  #! /bin/bash
  set -eo pipefail

  cargo build
  just build-harperjs
  just build-obsidian
  just test-vscode
  just test-harperjs
  just build-web

# Perform full format and type checking, build all projects and run all tests. Run this before pushing your code.
precommit:
  #! /bin/bash
  set -eo pipefail

  just check
  just test

  cargo doc
  cargo build
  cargo build --release
  cargo bench

  just build-harperjs
  just build-obsidian
  just build-web

# Install `harper-cli` and `harper-ls` to your machine via `cargo`
install:
  cargo install --path harper-ls
  cargo install --path harper-cli

# Run `harper-cli` on the Harper repository
dogfood:
  #! /bin/bash
  cargo build --release
  for file in `fd -e rs`
  do
    echo Linting $file
    ./target/release/harper-cli lint $file
  done

# Test everything.
test:
  cargo test
  just test-vscode
  just test-harperjs

# Use `harper-cli` to parse a provided file and print out the resulting tokens.
parse file:
  cargo run --bin harper-cli -- parse {{file}}

# Lint a provided file using Harper and print the results.
lint file:
  cargo run --bin harper-cli -- lint {{file}}

# Show the spans of the parsed tokens overlapped in the provided file.
spans file:
  cargo run --bin harper-cli -- spans {{file}}

# Add a noun to Harper's curated dictionary.
addnoun noun:
  #! /bin/bash
  DICT_FILE=./harper-core/dictionary.dict 

  cat $DICT_FILE | grep "^{{noun}}/"

  if [ $? -eq 0 ]
  then
    echo "That noun may already be in the dictionary."
    exit 0
  fi

  echo "{{noun}}/SM" >> $DICT_FILE

# Search Harper's curated dictionary for a specific word
searchdictfor word:
  cargo run --bin harper-cli -- words | rg {{word}}

# Find words in the user's `harper-ls/dictionary.txt` for words already in the curated dictionary.
userdictoverlap:
  #! /bin/bash
  USER_DICT_FILE="$HOME/.config/harper-ls/dictionary.txt"

  while read -r line; do
    just searchdictfor $line 2> /dev/null
  done < $USER_DICT_FILE

# Get the metadata associated with a particular word in Harper's dictionary as JSON.
getmetadata word:
  cargo run --bin harper-cli -- metadata {{word}}

bump-versions:
  #! /bin/bash
  set -eo pipefail

  cargo ws version --no-git-push --no-git-tag

  HARPER_VERSION=$(tq --file harper-core/Cargo.toml .package.version)

  cd "{{justfile_directory()}}/packages/harper.js"

  cat package.json | jq ".version = \"$HARPER_VERSION\"" > package.json.edited
  mv package.json.edited package.json

  cd "{{justfile_directory()}}/packages/vscode-plugin"

  cat package.json | jq ".version = \"$HARPER_VERSION\"" > package.json.edited
  mv package.json.edited package.json

  just format

  lazygit

# Enter an infinite loop of testing until a bug is found.
fuzz:
  #!/usr/bin/bash
  
  while true
  do
      QUICKCHECK_TESTS=100000 cargo test
      if [[ x$? != x0 ]] ; then
          exit $?
      fi
  done
