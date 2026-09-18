# Adding a language to Harper

A language is a directory under `harper-core/src/language/`. The build script
discovers it, generates the code that wires it into Harper, and gates it behind
a Cargo feature so that builds without it are unaffected. Only the Cargo
manifests have to be edited by hand.

Working on this as an agent? Read [`AGENTS.md`](./AGENTS.md) first.

## 1. Create the directory

```
harper-core/src/language/<lang>/
├── config.toml                 # required: what the build script reads
├── mod.rs                      # required: declares the submodules below
├── module.rs                   # required: the `LanguageModule` impl
├── dialects.rs                 # required: `<Name>Dialect` and `<Name>DialectFlags`
├── language_detection.rs       # the detector `module.rs` returns
├── lexing.rs                   # tokenizer, usually a thin wrapper over the English one
├── parsers/                    # `Plain<Name>`, the plain-prose parser
├── spell/                      # dictionary loading
├── linting/
│   ├── mod.rs                  # assembles the curated lint group
│   └── weir_rules/
│       ├── mod.rs              # one line: include!(env!("<LANG>_WEIR_RULE_LIST"))
│       └── *.weir              # one file per rule
├── dictionary.dict             # base words with property flags
├── annotations.json            # affix rules and flag -> metadata mapping
├── test_sources/               # optional: example prose, `*.md` or `*.txt`
└── stats.rs                    # optional
```

`config.toml` and `module.rs` are what the build script looks for — a directory
without both is ignored. The rest is convention, and only `module.rs` refers to
it, so the layout can differ if a language needs it to. English is the one that
does: it has no dictionary of its own here, because its dictionary lives at
`harper-core/`.

Everything is embedded with `include_str!()`, so these are compile-time paths.

Three things happen on their own once the files are in place: `stats.rs` is
picked up if present and `lang_stats <lang>` dispatches to it; the whole
`weir_rules` module, including a test per rule that runs its own `test` and
`allows` lines, is generated from the `.weir` files that are there; and a new
directory triggers regeneration without a `cargo clean`.

## 2. Write `config.toml`

```toml
[language]
name = "German"     # PascalCase; omit `feature` for a language that is always built
feature = "de"

[metadata]
confidence = 0.95   # detector order only, highest first

[[dialects]]
name = "Standard"   # must be accepted by `<Name>Dialect::try_from_abbr`
aliases = ["de", "german", "deutsch", "de-de", "de_de"]

[[dialects]]
name = "Austrian"
aliases = ["at", "austria", "austrian", "de-at", "de_at"]

[weir]               # optional; omit if `linting/weir_rules/` has no subdirectory
rules_subdirectory = "de"
```

The generator derives names from these values, so they have to line up:

| Config value | What must exist |
|---|---|
| `name = "German"` | `GermanDialect` in `dialects.rs`, `GermanModule` in `module.rs` |
| the directory name | `crate::language::german::…` paths |
| `feature = "de"` | the Cargo feature, and the user-dictionary suffix `-de` |
| a dialect `name` | a `try_from_abbr` value on the dialect enum |
| each `alias` | lower case, and unclaimed by any other language |

English's detector confidence is pinned to `0.30` by the generator whatever its
config says, so it is always tried last.

## 3. Implement `LanguageModule`

The trait in [`module.rs`](./module.rs) is the whole contract; each method says
what it is for. `slovak/module.rs` is the shortest complete example to copy.

Two things the compiler cannot tell you:

- **`Plain<Name>` must override `Parser::is_english` to return `false`.**
  Harper runs an English Brill tagger and a neural noun-phrase chunker over
  every document it builds. They are the larger half of that cost and produce
  nothing useful on another language: opting out roughly halves the time to
  build a document. The conformance suite fails if a parser forgets this.
- **`curated_lint_group` must actually assemble the group.** Returning
  `LintGroup::empty()` compiles, wires in cleanly, and silently checks nothing.

## 4. Declare the Cargo features

Five manifests, one line each:

```toml
# harper-core/Cargo.toml
<feature> = ["language-module"]
multilingual = [..., "<feature>"]

# harper-ls, harper-cli, harper-wasm, harper-desktop/src-tauri
<feature> = ["harper-core/<feature>"]
```

This stays manual so that each binary can ship its own language set. Run

```bash
just check-language-features
```

and it names every manifest still missing the feature. It discovers languages
from the `config.toml` files, so a new one is checked from the moment it exists.

### What the binaries actually ship

All four — `harper-cli`, `harper-ls`, `harper-wasm` and `harper-desktop` — carry
`default = ["multilingual"]`, so today every build contains every language. The
per-language features are the mechanism, not yet the practice.

That matters most for `harper-wasm`, because `just build-wasm` builds
`harper_wasm` with default features and `packages/chrome-plugin/vite.config.ts`
copies that binary into the extension. Every dictionary in the tree is therefore
in the extension, the desktop app, the website and `harper.js`. The WASM API
exposes the non-English dialects, but no front end in this repository offers
them in its UI, so what the extension gains from them today is size alone.
Measure it before changing anything here:

```bash
cd harper-wasm
wasm-pack build --target web --no-opt --out-dir /tmp/pkg-all  --out-name w
wasm-pack build --target web --no-opt --out-dir /tmp/pkg-en   --out-name w \
    --no-default-features --features english,typst,thesaurus
ls -l /tmp/pkg-all/w_bg.wasm /tmp/pkg-en/w_bg.wasm
```

The difference is data segments, not code, so it costs the extension memory and
download size rather than WebAssembly compile time — time both before claiming
either.

## 5. Check it

```bash
just check-languages        # feature wiring, per-language tests, dictionaries, coverage
cargo test -p harper-core --features all-languages --test language_conformance
just check-english-parity   # English behaves the same with and without languages
```

[`tests/language_conformance.rs`](../../tests/language_conformance.rs) walks
every language the build contains rather than naming any, so a new language is
covered by all of it as soon as it compiles: aliases resolve, the dictionary
loads and is non-empty, every prose format has a parser, the curated group has
uniquely named linters that describe themselves, lints land inside the document,
and an empty document produces none.

## What the build generates

`build.rs` writes these into this directory. They are committed, and editing
them by hand is pointless — change
[`build_lib/language_modules.rs`](../../build_lib/language_modules.rs) instead.

| File | Contents |
|---|---|
| `mod.rs` | module declarations and re-exports |
| `languages.rs` | `Language`, `LanguageFamily`, `parse_language`, `all_languages`, `language_aliases` |
| `registry.rs` | detection, dictionaries, parsers, lint groups, statistics |

Into `OUT_DIR`, uncommitted, it also writes each language's whole `weir_rules`
module, which is why that `mod.rs` is a single `include!`.

`harper-wasm/build.rs` likewise generates the WebAssembly `Dialect` enum and its
conversion to `Language`, including the fallbacks to English for languages left
out of a build, so `harper-wasm` needs no edit either.

## Feature flags

```toml
language-module = []                        # English; enabled by every language feature
multilingual    = ["de", "pt", "sk", "pl"]
de = ["language-module"]                    # likewise pt, sk, pl
all-languages   = ["multilingual"]
```

With no language feature at all, `pub mod language` does not exist and English
uses its original implementation untouched. Every consumer crate enables
`language-module` in its base `harper-core` dependency, so the English module is
always there; per-language features add languages on top.

## Tooling

Development recipes live in [`justfile`](./justfile):

```bash
just --list | grep language-
```

They cover spell-checking arbitrary text, inspecting word metadata, dictionary
validation and statistics, coverage and affix-efficiency analysis, Hunspell
comparison, test-template generation, and the per-language test suites. Each
recipe documents its own usage; the justfile is the source of truth, not this
file.

Two things the recipe help cannot say:

- **English is a special case**: its dictionary is not under `language/`, so
  some recipes do not apply to it.
- **Coverage needs a reference dictionary**: `just language-coverage <lang>`
  compares against `<lang>_dictionary.dict.gz` in the language directory. Only
  German ships one; the others are skipped.

## Improving a dictionary

1. Add or correct entries in `dictionary.dict`.
2. Add any new property or affix flags to `annotations.json`.
3. Check with `just language-meta <lang> "<word>"` or
   `just language-meta-text <lang> "<sentence>"`.
4. Find gaps against Hunspell: `just language-hunspell <lang> "<text>"`.
5. Confirm coverage has not regressed: `just language-coverage <lang>`.

Property flags are single characters, and in German 21 letters are
simultaneously a property *and* an affix rule, so a letter flag can generate
surface forms you did not ask for. Only digits and punctuation are free — use
digits for new property flags.

Prefer dictionary data over Rust constants: a word list in a `const &[&str]` is
almost always misplaced. See [`german/README.md`](./german/README.md) for the
exceptions and why they are exceptions.
