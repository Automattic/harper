# Harper Language Testing Framework (`harper-lang-test`)

A development binary for iterating on language dictionaries and annotations
without recompiling Harper itself. It loads `dictionary.dict` and
`annotations.json` from disk at runtime, so a dictionary edit is testable
immediately.

This crate declares its own `[workspace]`, so `cargo test --workspace` does not
build it. `just check-languages` runs `cargo check` against it so it cannot rot
unnoticed.

## Usage

You normally do not invoke the binary directly — the `just language-*` recipes
wrap it and pass the right paths:

```bash
just --list | grep language-
```

To build it once:

```bash
just language-build
```

The recipes cover: spell-checking arbitrary text, showing metadata for a word or
a whole sentence, dictionary self-tests, coverage and affix-efficiency analysis,
and Hunspell comparison.

## Direct invocation

```bash
./target/release/harper-lang-test \
  --language german \
  --dict        ../german/dictionary.dict \
  --annotations ../german/annotations.json \
  --text "die mondlandung ist wieder fehlgeschlagen"
```

Options:

| Flag | Meaning |
|------|---------|
| `--language <name>` | Language directory name (`german`, `portuguese`, ...). Required. |
| `--dict <path>` | Word list. Defaults to `../../language/<lang>/dictionary.dict`. |
| `--annotations <path>` | Annotation file. Defaults alongside the dictionary. |
| `--text <str>` | Spell-check this text. |
| `--word <str>` | Show metadata for a single word. |
| `--metadata` | With `--text`, show metadata for every word. |
| `--test` | Run the built-in dictionary self-tests. |
| `--hunspell` | Compare results against system Hunspell. |
| `--coverage` | Measure recognition against `--expanded-dict`. |
| `--expanded-dict <path>` | Gzipped reference word list for `--coverage`. |
| `--sample-size <n>` | Words sampled for coverage. `0` (default) checks every word. |
| `--min-coverage <pct>` | Exit non-zero if coverage falls below this. |

## Coverage

`--coverage` compares Harper's expanded dictionary against an external reference
list (`<lang>_dictionary.dict.gz`). Only German currently ships one.

By default **every** word in the reference list is checked — 258k words for
German, about two seconds. That is both representative and reproducible, so a
change in the number means a real change in the dictionary.

`--sample-size <n>` takes a random sample instead, for a quicker estimate while
iterating. Do not gate CI on one: a 10,000-word sample carries roughly ±0.5% of
sampling noise.

Note that sampling also stops reading early once it has enough lines. Because
reference word lists are sorted alphabetically, a small sample is drawn from the
front of the alphabet rather than the whole list, so its absolute value is not
comparable to the exhaustive figure.

## German specifics

For German the tool builds a `CompoundAwareDictionary` rather than a plain FST,
so compound recognition (`Donaudampfschifffahrtsgesellschaft`) is exercised the
same way it is at runtime.
