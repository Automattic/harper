# Format entire project
alias fmt := format
format:
  cargo fmt  
  pnpm format

# Build the shared component library
build-components:
  #!/usr/bin/env bash
  set -eo pipefail

  cd "{{justfile_directory()}}/packages/components"
  pnpm install --engine-strict=false
  pnpm build

# Build the WebAssembly module
build-wasm:
  #!/usr/bin/env bash
  cd "{{justfile_directory()}}/harper-wasm"
  if [ "${DISABLE_WASM_OPT:-0}" -eq 1 ]; then
    wasm-pack build --target web --no-opt
  else
    wasm-pack build --target web
  fi

# Build `harper.js` with all size optimizations available.
build-harperjs: build-wasm 
  #!/usr/bin/env bash
  set -eo pipefail

  # Removes a duplicate copy of the WASM binary if Vite is left to its devices.
  perl -pi -e 's/new URL\(.*\)/new URL()/g' "{{justfile_directory()}}/harper-wasm/pkg/harper_wasm.js"

  cd "{{justfile_directory()}}/packages/harper.js"
  pnpm install
  pnpm build

  # Generate API reference
  ./docs.sh

# Build the browser lint framework module
build-lint-framework: build-harperjs
  #!/usr/bin/env bash
  set -eo pipefail

  cd "{{justfile_directory()}}/packages/lint-framework"
  pnpm install
  pnpm build

test-harperjs: build-harperjs
  #!/usr/bin/env bash
  set -eo pipefail

  pnpm install
  cd "{{justfile_directory()}}/packages/harper.js"
  pnpm playwright install
  pnpm test

  # Test runnable examples
  cd "{{justfile_directory()}}/packages/harper.js/examples/commonjs-simple"
  pnpm start

test-obsidian: build-obsidian
  #!/usr/bin/env bash
  set -eo pipefail

  pnpm install
  cd "{{justfile_directory()}}/packages/obsidian-plugin"
  pnpm playwright install
  pnpm test

dev-wp: build-harperjs
  #!/usr/bin/env bash

  set -eo pipefail

  cd "{{justfile_directory()}}/packages/wordpress-plugin"
  pnpm install
  pnpm wp-now start &
  pnpm start 

# Build the WordPress plugin
alias build-wordpress := build-wp
build-wp: build-harperjs
  #!/usr/bin/env bash
  set -eo pipefail

  cd "{{justfile_directory()}}/packages/wordpress-plugin"
  pnpm install
  pnpm build
  pnpm plugin-zip

# Compile the website's dependencies and start a development server. Note that if you make changes to `harper-wasm`, you will have to re-run this command.
dev-web: build-harperjs build-lint-framework build-components
  #!/usr/bin/env bash
  set -eo pipefail

  cd "{{justfile_directory()}}/packages/web"
  pnpm install
  pnpm dev

# Build the Harper website.
build-web: build-harperjs build-lint-framework build-components
  #!/usr/bin/env bash
  set -eo pipefail
  
  cd "{{justfile_directory()}}/packages/web"
  pnpm install
  pnpm build

# Build the Harper Obsidian plugin.
build-obsidian: build-harperjs
  #!/usr/bin/env bash
  set -eo pipefail
  
  cd "{{justfile_directory()}}/packages/obsidian-plugin"

  pnpm install
  pnpm build

  zip harper-obsidian-plugin.zip manifest.json main.js

# Build the Chrome extension.
alias build-chrome := build-chrome-plugin
alias build-chrome-extension := build-chrome-plugin
build-chrome-plugin: build-harperjs build-lint-framework build-components
  #!/usr/bin/env bash
  set -eo pipefail
  
  cd "{{justfile_directory()}}/packages/chrome-plugin"

  pnpm install 
  pnpm zip-for-chrome

# Start a development server for the Chrome extension.
alias dev-chrome := dev-chrome-plugin
alias dev-chrome-extension := dev-chrome-plugin
dev-chrome-plugin: build-harperjs build-lint-framework build-components
  #!/usr/bin/env bash
  set -eo pipefail
  
  cd "{{justfile_directory()}}/packages/chrome-plugin"

  pnpm install 
  pnpm dev

# Build the Firefox extension.
alias build-firefox := build-firefox-plugin
alias build-firefox-extension := build-firefox-plugin
build-firefox-plugin: build-harperjs build-lint-framework build-components
  #!/usr/bin/env bash
  set -eo pipefail
  
  cd "{{justfile_directory()}}/packages/chrome-plugin"

  pnpm install 
  pnpm zip-for-firefox

alias test-chrome := test-chrome-plugin
alias test-chrome-extension := test-chrome-plugin
test-chrome-plugin: build-chrome-plugin
  #!/usr/bin/env bash
  set -eo pipefail

  pnpm install
  cd "{{justfile_directory()}}/packages/chrome-plugin"
  pnpm playwright install

  # For environments without displays like CI servers or containers
  if [[ "$(uname)" == "Linux" ]] && [[ -z "$DISPLAY" ]]; then
    xvfb-run --auto-servernum pnpm test --project chromium
  else
    pnpm test --project chromium
  fi


alias test-firefox := test-firefox-plugin
alias test-firefox-extension := test-firefox-plugin
test-firefox-plugin: build-firefox-plugin
  #!/usr/bin/env bash
  set -eo pipefail

  pnpm install
  cd "{{justfile_directory()}}/packages/chrome-plugin"
  pnpm playwright install
  # For environments without displays like CI servers or containers
  if [[ "$(uname)" == "Linux" ]] && [[ -z "$DISPLAY" ]]; then
    xvfb-run --auto-servernum pnpm test --project firefox
  else
    pnpm test --project firefox 
  fi

# Run VSCode plugin unit and integration tests.
alias test-vscode-extension := test-vscode
test-vscode:
  #!/usr/bin/env bash
  set -eo pipefail

  ext_dir="{{justfile_directory()}}/packages/vscode-plugin"
  bin_dir="${ext_dir}/bin"

  if ! [[ -d "$bin_dir" ]]; then
    mkdir "$bin_dir"
  fi

  echo Building binaries
  cargo build --release -q

  cp "{{justfile_directory()}}/target/release/harper-ls"* "$bin_dir"

  cd "$ext_dir"

  pnpm install
  # For environments without displays like CI servers or containers
  if [[ "$(uname)" == "Linux" ]] && [[ -z "$DISPLAY" ]]; then
    xvfb-run --auto-servernum pnpm test
  else
    pnpm test
  fi

  # Over time, VSCode test versions take up space that can be hard to track down
  if [[ -d .vscode-test ]]; then
    all_versions=$(ls -1 .vscode-test | grep "^vscode-" | sort -V)
    latest_version=$(echo "$all_versions" | tail -n 1)
    old_versions=$(echo "$all_versions" | sed '$d')  # Delete last line instead
    if [[ -n "$old_versions" ]]; then
      count=$(echo "$old_versions" | wc -l)
      echo "$old_versions" | xargs -I {} rm -rf .vscode-test/{}
      echo "✓ Deleted $count old VSCode versions, keeping $latest_version"
    else
      echo "✓ No old versions to clean (keeping $latest_version)"
    fi
  fi

# Build and package the Visual Studio Code extension.
# If `target` is passed, it is assumed that `harper-ls` has been compiled beforehand and is in `packages/vscode-plugin/bin`. This is used in CI.
package-vscode target="":
  #!/usr/bin/env bash
  set -eo pipefail

  ext_dir="{{justfile_directory()}}/packages/vscode-plugin"
  bin_dir="${ext_dir}/bin"

  cp LICENSE "$ext_dir"

  if [[ -z "{{target}}" ]]; then
    echo Building binaries
    cargo build --release -q

    if ! [[ -d "$bin_dir" ]]; then
      mkdir "$bin_dir"
    fi

    cp "{{justfile_directory()}}/target/release/harper-ls"* "$bin_dir"
  fi

  cd "$ext_dir"

  pnpm install
  if [[ -n "{{target}}" ]]; then
    pnpm package --target {{target}}
  else
    pnpm package
  fi

update-vscode-linters:
  #!/usr/bin/env bash
  set -eo pipefail

  linters=$(
    cargo run --bin harper-cli -- config |
      jq 'with_entries(.key |= "harper.linters." + . |
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
        with_entries(select(.key | startswith("harper.linters") | not))
      end
    )' package.json
  )

  jq --argjson linters "$linters" \
    '.contributes.configuration.properties += $linters' <<< \
    "$manifest_without_linters" > \
    package.json
  just format

# Run Rust formatting and linting.
check-rust: auditdictionary
  #!/usr/bin/env bash
  set -eo pipefail

  cargo fmt -- --check
  cargo clippy -- -Dwarnings -D clippy::dbg_macro -D clippy::needless_raw_string_hashes

  cargo hack check --each-feature

# Perform format and type checking.
check: check-rust check-js build-web

check-js: build-harperjs build-lint-framework build-components
  #!/usr/bin/env bash
  set -eo pipefail

  pnpm install
  pnpm check

  # Needed because Svelte has special linters
  cd "{{justfile_directory()}}/packages/web"
  pnpm check

# Populate build caches and install necessary local tooling (tools callable via `pnpm run <tool>`).
setup: build-harperjs test-harperjs test-vscode build-web build-wp build-obsidian build-chrome-plugin

# Perform full format and type checking, build all projects and run all tests. Run this before pushing your code.
precommit: check test build-harperjs build-obsidian build-web build-wp build-firefox-plugin build-chrome-plugin 
  #!/usr/bin/env bash
  set -eo pipefail

  echo Building binaries
  cargo build --all-targets -q

# Install `harper-cli` and `harper-ls` to your machine via `cargo`
install:
  cargo install --path harper-ls --locked 
  cargo install --path harper-cli --locked 

# Run `harper-cli` on the Harper repository
dogfood:
  #!/usr/bin/env bash
  cargo build --release
  for file in `fd -e rs`
  do
    echo Linting $file
    ./target/release/harper-cli lint $file
  done

test-rust:
  echo Running all Rust tests
  cargo test -q

# Test everything.
test: test-rust test-harperjs test-vscode test-obsidian test-chrome-plugin test-firefox-plugin

# Use `harper-cli` to parse a provided file and print out the resulting tokens.
parse file:
  cargo run --bin harper-cli -- parse {{file}}

# Lint provided inputs using Harper and print the results.
# The inputs can be files, directories, or a string on the command line.
# If no inputs are provided, lint stdin.
lint *inputs:
  cargo run --bin harper-cli -- lint {{inputs}}

# Show the spans of the parsed tokens overlapped in the provided file.
spans file:
  cargo run --bin harper-cli -- spans {{file}}

# Add a noun to Harper's curated dictionary.
alias add-noun := addnoun
addnoun noun:
  #!/usr/bin/env bash
  DICT_FILE=./harper-core/dictionary.dict 

  cat $DICT_FILE | grep "^{{noun}}/"

  if [ $? -eq 0 ]
  then
    echo "That noun may already be in the dictionary."
    exit 0
  fi

  # 'g': possessive -'s suffix for both common and proper nouns
  flags='g'

  # If the first letter is uppercase, treat it as a proper noun
  if [[ "{{noun}}" =~ ^[A-Z] ]]; then
    # 'O': proper noun, usually no plural
    flags+='O'
  else
    # 'N': (common) singular noun, 'S': plural -(e)s
    flags+='NS'
  fi

  # Echo the noun with its flags to the dictionary file
  [[ -s $DICT_FILE && -n $(tail -c1 "$DICT_FILE") ]] && echo >> "$DICT_FILE"
  echo "{{noun}}/$flags" >> "$DICT_FILE"

# Search Harper's curated dictionary for a specific word
alias search-dict-for := searchdictfor
searchdictfor word:
  #!/usr/bin/env bash
  if command -v rg > /dev/null; then
    cargo run --bin harper-cli -- words | rg {{word}}
  else
    cargo run --bin harper-cli -- words | grep {{word}}
  fi

# Find words in the user's `harper-ls/dictionary.txt` for words already in the curated dictionary.
alias user-dict-overlap := userdictoverlap
userdictoverlap:
  #!/usr/bin/env bash
  USER_DICT_FILE="$HOME/.config/harper-ls/dictionary.txt"

  while read -r line; do
    just searchdictfor $line 2> /dev/null
  done < $USER_DICT_FILE

# Get the metadata associated with one or more words in Harper's dictionary as JSON.
alias get-metadata := getmetadata
getmetadata *words:
  cargo run --bin harper-cli -- metadata {{words}}

alias get-metadata-brief := getmetadata-brief
getmetadata-brief *words:
  cargo run --bin harper-cli -- metadata --brief {{words}}

# Get all the forms of a word using the affixes.
alias get-forms := getforms
getforms word:
  cargo run --bin harper-cli -- forms {{word}}

# Get a random sample of words from Harper's dictionary and list all forms of each.
alias sample-forms := sampleforms
sampleforms count:
  #!/usr/bin/env bash
  set -eo pipefail
  DICT_FILE=./harper-core/dictionary.dict 
  # USER_DICT_FILE="$HOME/.config/harper-ls/dictionary.txt"

  if [ "{{count}}" -eq 0 ]; then
    exit 0
  fi

  total_lines=$(wc -l < $DICT_FILE)
  
  # Cross-platform random line selection
  if command -v shuf >/dev/null 2>&1; then
    words=$(shuf -n "{{count}}" "$DICT_FILE")
  elif command -v jot >/dev/null 2>&1; then
    words=$(jot -r "{{count}}" 1 "$total_lines" | while read -r line_num; do \
      sed -n "$line_num"p "$DICT_FILE"; \
    done)
  else
    echo "Error: Neither 'shuf' nor 'jot' found. Cannot generate random words." >&2
    exit 1
  fi
  
  cargo run --bin harper-cli -- forms $words

bump-versions: update-vscode-linters
  #!/usr/bin/env bash
  set -eo pipefail

  cargo ws version --no-git-push --no-git-tag --force '*'

  HARPER_VERSION=$(tq --raw --file harper-core/Cargo.toml .package.version)

  cd "{{justfile_directory()}}/packages/harper.js"

  cat package.json | jq ".version = \"$HARPER_VERSION\"" > package.json.edited
  mv package.json.edited package.json

  cd "{{justfile_directory()}}/packages/vscode-plugin"

  cat package.json | jq ".version = \"$HARPER_VERSION\"" > package.json.edited
  mv package.json.edited package.json

  cd "{{justfile_directory()}}/packages/chrome-plugin"

  cat package.json | jq ".version = \"$HARPER_VERSION\"" > package.json.edited
  mv package.json.edited package.json

  cd "{{justfile_directory()}}/packages/obsidian-plugin"

  cat package.json | jq ".version = \"$HARPER_VERSION\"" > package.json.edited
  mv package.json.edited package.json

  just format

  lazygit

# Enter an infinite loop of property testing until a bug is found.
fuzz:
  #!/usr/bin/env bash
  
  while true
  do
      QUICKCHECK_TESTS=100000 cargo test
      if [[ x$? != x0 ]] ; then
          exit $?
      fi
  done

alias register-linter := registerlinter
registerlinter module name:
  #!/usr/bin/env bash

  D="{{justfile_directory()}}/harper-core/src/linting"

  sed -i "/pub use an_a::AnA;/a pub use {{module}}::{{name}};" "$D/mod.rs"
  sed -i "/use super::an_a::AnA;/a use super::{{module}}::{{name}};" "$D/lint_group.rs"
  sed -i "/insert_expr_rule!(ChockFull, true);/a \ \ \ \ insert_struct_rule!({{name}}, true);" "$D/lint_group.rs"
  just format

# Print annotations and their descriptions from annotations.json
alias printaffixes := printannotations
alias getannotations := printannotations
alias listannotations := printannotations
alias showannotations := printannotations
printannotations:
  #! /usr/bin/env node
  const affixesData = require('{{justfile_directory()}}/harper-core/annotations.json');
  const allEntries = {
    ...affixesData.affixes || {},
    ...affixesData.properties || {}
  };
  
  // Calculate the maximum description length for alignment
  const entries = Object.entries(allEntries);
  const maxDescLength = entries.reduce((max, [flag, fields]) => {
    const description = fields['#'] || '';
    const lineLength = flag.length + 2 + description.length; // flag + ': ' + description
    return Math.max(max, lineLength);
  }, 0);
  
  entries.sort((a, b) => a[0].localeCompare(b[0])).forEach(([flag, fields]) => {
    const description = fields['#'] || '';
    const comment = fields['//'] || null;
    if (description) {
      const line = `${flag}: ${description}`;
      const padding = ' '.repeat(Math.max(1, maxDescLength - line.length + 2));
      console.log(line + (comment ? `${padding}// ${comment}` : ''));
    }
  });
  
  console.log('Available letters for new flags:', [...Array.from({length: 26}, (_, i) => 
    [String.fromCharCode(65 + i), String.fromCharCode(97 + i)]
  ).flat()].filter(letter => !Object.keys(allEntries).includes(letter)).sort().join(' '));
  console.log('Available digits for new flags:', [...Array.from({length: 10}, (_, i) => 
    String(i)
  )].filter(digit => !Object.keys(allEntries).includes(digit)).sort().join(' '));
  console.log('Available symbols for new flags:',
    [...Array.from('!"#$%&\'()*+,-./:;<=>?@\[\\\]\^_`{|}~')]
  .filter(symbol => !Object.keys(allEntries).includes(symbol)).sort().join(' '));
  console.log('Available Latin-1 characters for new flags:'); 
  [...Array.from({length: 256-160}, (_, i) => String.fromCharCode(160 + i))]
    .filter(char => !Object.keys(allEntries).includes(char) && char.charCodeAt(0) !== 160 && char.charCodeAt(0) !== 173)
    .sort()
    .join(' ')
    .match(/.{1,64}/g)
    .forEach(line => console.log('  ' + line));
    
# Get the most recent changes to the curated dictionary. Includes an optional argument to specify the number of commits to look back. Defaults to 1.
newest-dict-changes *numCommits:
  #! /usr/bin/env node

  const { exec } = require('child_process');

  const DICT_FILE = 'harper-core/dictionary.dict';

  const [RST, BOLD, DIM, ITAL, NORM] = [0, 1, 2, 3, 22].map(c => `\x1b[${c}m`);
  const [RED, GRN, YLW, BLU, MGN, CYN, WHT] = [1, 2, 3, 4, 5, 6, 7].map(c => `\x1b[${30+c}m`);

  const argv = [...process.argv];

  const [showHashes, showDiff] = ["--show-hashes", "--show-diff"].map(flag => argv.includes(flag) && argv.splice(argv.indexOf(flag), 1));

  // uncomment first line to use in justfile, comment out second line to use standalone
  const numCommits = "{{numCommits}}" || 1;
  // const numCommits = argv[2] || 1;

  // Command to get the last commit hash that modified the specified file
  const hashCommand = `git log --no-merges -n ${numCommits} --format="%H" -- ${DICT_FILE}`;
  console.log(`${MGN}${BOLD}GET HASHES${NORM}: ${hashCommand}${RST}`);

  // Execute the command to get the hash
  exec(hashCommand, (error, hashString, stderr) => {
    if (error) return console.error(`Error executing command: ${error.message}`);
    if (stderr) return console.error(`stderr: ${stderr}`);

    // avoid empty last line
    const longHashes = hashString.trim().split('\n');
    if (showHashes) console.log(longHashes.length, longHashes);

    if (longHashes.length < 1) {
      console.error('No hash(es) returned. Exiting.');
      process.exit(1);
    }

    // keep the last line and second last if there's more than one hash
    const [hash2, hash1] = longHashes.slice(-2).map((h) => h.substring(0, 7));

    // Command to get the word-level diff using the retrieved hash, using either one or two hashes
    const hashes = longHashes.length == 1 ? `${hash2}` : `${hash1} ${hash2}`;
    const diffCommand = `git diff --word-diff --no-color --unified=0 ${hashes} -- ${DICT_FILE}`;
    console.log(`${MGN}${BOLD}GET DIFF${NORM}: ${diffCommand}${RST}`);

    // Execute the diff command with a large buffer to avoid failing to handle large diffs such as:
    // git diff --word-diff --no-color --unified=0 0761702 baeb08e -- harper-core/dictionary.dict
    exec(diffCommand, { maxBuffer: 2048 * 1024 }, (diffError, diffString, diffStderr) => {
      if (diffError) {
        console.error(`Error executing diff command: ${diffError.message}`);
        return;
      }
      if (diffStderr) return console.error(`stderr: ${diffStderr}`);

      if (showDiff) console.log(`DIFFSTART\n${diffString}\nDIFFEND`);

      // uncomment first line to use in justfile, comment out second line to use standalone
      const affixes = require('{{justfile_directory()}}/harper-core/annotations.json').affixes;
      // const affixes = require('./harper-core/annotations.json').affixes;

      diffString.split("\n").forEach(line => {
        const match = line.match(/^(?:\[-(.*?)-\])?(?:\{\+(.*?)\+\})?$/);
        if (match) {
          let [, before, after] = match;

          if (before && after) {
            // An entry changed
            const [[oldword, oldaff], [newword, newaff]] = [before, after].map(e => e.split('/'));
            if (oldword === newword) {
              if (oldaff !== newaff) {
                const [oldRest, newRest] = [oldaff, newaff].map(aff => aff ? `${DIM}/${aff}${RST}`: '');
                console.log(`${BOLD}${CYN}CHG${RST} # ${oldword}${oldRest} -> ${newRest}`);
                const [oldNorm, newNorm] = [oldaff, newaff].map(a => a ? a.split(''): [])
                                                           .map(a => new Set(a))
                                                           .map(a => Array.from(a))
                                                           .map(a => a.sort());
                const removed = oldNorm.filter(o => !newNorm.includes(o));
                const added = newNorm.filter(n => !oldNorm.includes(n));
                const [addStr, remStr] = [added, removed]
                  .map(a => a.map(a => `    ${BOLD}${ITAL}${a}${RST} -> ${ (affixes[a] && affixes[a]['#']) || '???' }`)
                             .join('\n')
                  );
                if (removed.length > 0) console.log(`${RED}  ${BOLD}REMOVED${RST}:\n${remStr}`);
                if (added.length > 0) console.log(`${GRN}  ${BOLD}ADDED${RST}:\n${addStr}`);
              } else {
                // should never happen
                console.log(`${YLW} ?NO AFFIX CHG? '${oldaff}' -> '${newaff}'${RST}`);
              }
            } else {
              // The word changed rather than its affixes
              console.log(`${YLW}  ${BOLD}CHANGED${RST} ${RED}${oldword}${RST} -> ${GRN}${newword}${RST}`);
            }
          } else if (before || after) {
            // An entry was added or removed
            const [entry, symbol, action, colour] = before ? [before, "-", 'DEL', RED] : [after, "+", 'ADD', GRN];
            const [word, affix] = entry.split('/');
            console.log(`${colour}${BOLD}${action}${RST} ${symbol} ${word}${ affix ? `${DIM}/${affix}` : '' }${RST}`);
          }
        }
      });
    });
  });

# Print the input string or file with nominal phrases highlighted. These are generated using Harper's chunker.
alias get-nps := getnps
alias get-nominal-phrases := getnps
alias get-noun-phrases := getnps
getnps text:
  cargo run --bin harper-cli -- nominal-phrases "{{text}}"

# Suggest annotations for a potential new property annotation
alias suggest-annotation := suggestannotation
suggestannotation input:
  #! /usr/bin/env node
  const affixesData = require('{{justfile_directory()}}/harper-core/annotations.json');
  const allEntries = {
    ...affixesData.affixes || {},
    ...affixesData.properties || {}
  };
  
  // Get all used flags
  const usedFlags = new Set(Object.keys(allEntries));
  
  // Process input string and check both cases
  const input = '{{input}}';
  const normalizedInput = input.replace(/\s/g, '');
  const uniqueChars = [...new Set(normalizedInput.toUpperCase() + normalizedInput.toLowerCase())];
  
  console.log(`Checking input: "${input}"\n${'='.repeat(50)}`);
  
  // Check each character in input
  const availableChars = [...new Set(uniqueChars)]
    .filter(char => !usedFlags.has(char));
  
  if (availableChars.length > 0) {
    console.log(`These characters of "${input}" are available to use for new annotations:`);
    availableChars.forEach(char => console.log(`  '${char}' (${char.charCodeAt(0)})`));
  } else {
    const inputChars = new Set(normalizedInput.toLowerCase() + normalizedInput.toUpperCase());
    const renamable = Object.entries(allEntries)
      .filter(([flag, entry]) => entry.rename_ok && inputChars.has(flag))
      .sort((a, b) => a[0].localeCompare(b[0]));
    
    if (renamable.length > 0) {
      console.log(`None of the characters of "${input}" are available to use for new annotations, but these ones are OK to be moved to make way for new annotations:`);
      renamable.forEach(([flag, entry]) => {
        console.log(`  '${flag}': ${entry['#'] || 'No description'}${entry['//'] ? ` (${entry['//']})` : ''}`);
      });
    } else {
      console.log(`None of the characters of "${input}" are available to use for new annotations, and none of them are OK to be moved to make way for new annotations.`);
    }
  }

# Audit the curated dictionary for any issues.
alias audit-dictionary := auditdictionary
alias auditdict := auditdictionary
alias audit-dict := auditdictionary
auditdictionary DIR="harper-core":
  cargo run --bin harper-cli -- audit-dictionary {{DIR}}

alias run-snapshots := runsnapshots
alias test-snapshots := runsnapshots
alias test-pos-tagger := runsnapshots
alias test-pos-tags := runsnapshots
alias test-pos-tagging := runsnapshots
runsnapshots:
  #!/usr/bin/env bash
  set -eo pipefail

  cd harper-core
  cargo test -- test_pos_tagger test_most_lints
