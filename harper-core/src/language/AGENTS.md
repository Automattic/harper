# Harper Language Modules — Agent Guide

Guidance for working on Harper's multilingual support. The root `AGENTS.md` still
applies; this file covers everything under `harper-core/src/language/`.

`README.md` in this directory is the procedure for adding a language. This file
covers the rules that are easy to violate by accident.

## The one rule

**Language-specific code lives in `harper-core/src/language/`, or is emitted by
the build system. Nothing else.**

English is the deliberate exception: it stays on its original implementation
(`dict_word_metadata.rs`, `language_detection.rs`, `linting/`) so the branch
merges cleanly from upstream. `harper-core/src/language/english/` is a thin
adapter over that code, not a copy of it.

Concretely, before you add anything outside this directory, check:

- Does it name a grammatical concept English does not have (case, gender,
  declension, compounding)? It belongs in a language module.
- Does it widen visibility in core? Prefer `pub(crate)`. Only use `pub` when an
  out-of-crate consumer genuinely needs it, and gate it on `language-module` so
  the English-only build is unchanged (see `spell/mod.rs` for `rune`).
- Does it add a field to a shared struct? See "Morphology" below.

Verify with:

```bash
git diff master --stat -- harper-core/src/ ':(exclude)harper-core/src/language'
```

Anything you add there is a future merge conflict. Justify it or move it.

## Performance

**A language must not cost the others anything, and should cost itself as little
as it can.** Check it, do not assume it:

```bash
just language-perf german     # startup, lint time and peak memory, against English
```

- Run it before and after any change to a dictionary, an affix, `annotations.json`
  or the spell path. A change that raises startup or peak memory needs a reason.
- Startup is the dictionary build, paid once per process; lint time is the rules
  and the spell suggestions. They have different causes, so read them apart.
- Fault-dense text inflates lint time (each unknown word starts a suggestion
  search). Compare like with like, or pass `--sample` a clean text.
- Do not write measured figures into READMEs. They go stale; the recipe does not.
- To find who owns the memory, use `valgrind --tool=massif`, not guesswork (see
  "Dictionary memory" below).

### The German dictionary pack

Most of German's startup is expanding the word list into forms. `just
language-bake german` writes the expanded dictionary to a pack file; a program
that finds it (`HARPER_GERMAN_PACK`, or `harper-german.pack` next to the
executable) reads it instead. Without a pack, or with one built from another
word list or version, it expands as before and gets the same dictionary.

- The pack is an accelerator, never a requirement. Keep the expansion path working.
- Do not commit a pack: it is a build artifact, 30 MB, and the release job makes it
  from the same commit as the binary. The stamp in its header refuses any other.
- The stamp holds the crate version, the word list, the annotations and the source
  of the files that expand them (`pack_stamp` in `german_dict.rs`). Change any of
  them and an old pack is refused, so it can not serve stale forms. If the expansion
  starts to read another file, add it to that list.
- To compare, run `just language-perf german` with and without the pack beside the
  executable; the test `a_pack_reads_back_as_the_dictionary_it_was_written_from`
  checks every word and its metadata.

## Morphology

Inflectional features live in `morphology.rs`, not in `dict_word_metadata.rs`.
That file carries exactly one feature-gated field:

```rust
#[cfg(feature = "language-module")]
pub morphology: Option<crate::language::morphology::Morphology>,
```

Use the `MorphologyExt` trait for access (`get_noun_gender`, `get_determiner_case`,
...). In `annotations.json`, features go under a `morphology` key:

```json
"M": { "metadata": { "noun": {}, "morphology": { "noun": { "gender": "Masculine" } } } }
```

Agreement features are grouped **per part of speech** on purpose. A headword is
routinely several parts of speech at once — German `der` is both a determiner and
(per the dictionary) a noun. Flattening `gender` onto `Morphology` would let the
noun's gender read back as the determiner's, and the agreement linters compare
exactly those two values. Flattening would silently disable them.

These features are not German-specific: Slovak and Polish need case and gender
too. Put shared features in `morphology.rs`, not under `german/`.

## Generated files — do not edit

`build.rs` (via `build_lib/`) writes these from each language's `config.toml`:

- `language/mod.rs`
- `language/languages.rs`
- `language/registry.rs`
- each language's `linting/weir_rules` module body (into `OUT_DIR`, not
  committed), which is why that `mod.rs` is a single `include!`
- the `Dialect` enum in `harper-wasm` (into `OUT_DIR`, not committed)

Edits are overwritten on the next build. To change what they contain, edit
`harper-core/build_lib/language_modules.rs` or `build_lib/weir_rules.rs`.

## Adding a language

Most of it is discovery-driven: create `language/<lang>/` with a `config.toml`
and the files listed in `README.md`, implement `LanguageModule` in `module.rs`,
and the build system wires up the rest.

The Cargo manifests are the only manual part:

1. `harper-core/Cargo.toml`: add `<lang> = ["language-module"]` and add `<lang>`
   to `multilingual`.
2. Forward the feature in `harper-ls`, `harper-cli`, `harper-wasm` and
   `harper-desktop/src-tauri` `Cargo.toml`.

`just check-language-features` discovers languages from `config.toml` and names
every manifest that is still missing one. `lang_stats` needs no arm of its own:
the dispatch is generated for every language that ships a `stats.rs`.

Two mistakes the compiler will not catch, and `tests/language_conformance.rs`
will: a `Plain<Name>` parser that does not override `Parser::is_english` to
`false` (it then pays for the English Brill tagger and neural chunker, which are
the larger half of document construction, for tags no linter reads), and a
`curated_lint_group` that returns `LintGroup::empty()` and so checks nothing.

`harper-wasm` needs no edit. Its `Dialect` enum, the per-language dialect
imports, and `impl From<Dialect> for Language` are generated by
`harper-wasm/build.rs` from `config.toml`. Dialects convert through
`try_from_abbr`, which takes the dialect name exactly as written in the config
(Portuguese `PT`/`BR`/`AO` → `European`/`Brazilian`/`African`), so the generator
never needs the Rust variant names. The `#[cfg(not(feature = ...))]` fallback
arms that map a missing language to English are generated too — omitting one by
hand produced a non-exhaustive match only in builds that excluded that language,
which is exactly the kind of break CI is least likely to catch.

## Dictionary work

`dictionary.dict` holds base words with property flags; `annotations.json` maps
flags to metadata and defines affix rules.

**The flag namespace is nearly exhausted, and it is shared.** 21 letters are
simultaneously a property *and* an affix rule, so tagging a word with a letter
flag can generate unintended surface forms — adding the determiner flag `D` to
`mein` would also produce `meinung`. Only digits and punctuation are free. Use
digits for new property flags; English already does this (`4` = abbreviation).

**Order the replacements of an affix by the spelling you want kept.** Two forms
that differ only in case (`Lehrerinnen`, `LehrerInnen`) are one dictionary entry,
and the one an affix produces first is its spelling; the other is accepted but
never offered as a correction. Both are produced by the same flag (`K`, `L`), so
list the plain spelling's rules before the interior-capital ones. Expansion walks a
flag's replacements in that order (`new_word_order` in `spell/rune/attribute_list.rs`);
it used to walk a hash map, which made the choice differ from run to run.

Prefer dictionary data over Rust constants. A word list in a `const &[&str]` is
almost always misplaced — see `german/README.md` for the three that legitimately
stay in code and why.

When a linter needs a fast membership test over dictionary-flagged words, build
the set once from the base word list in a `LazyLock`, as
`german/spell/lexical_classes.rs` does. Do **not** call `get_word_metadata` per
token as a pre-filter: `CompoundAwareDictionary` takes a global mutex and runs a
compound decomposition on every miss.

## Tests

Language tests belong inside the language module (`german/tests/`,
`slovak/tests.rs`), not in `harper-core/tests/`. A test in `harper-core/tests/`
is an out-of-crate consumer, so it can only reach `pub` API — which is how
`spell::rune` ended up public. Anything touching crate internals must live in the
module.

Integration tests that remain in `harper-core/tests/` must be feature-gated with
`#![cfg(feature = "<lang>")]` at the top of the file.

## Measuring whether a rule is any good

A lint count on its own says nothing. A guard that deletes a thousand reports
and a rule that fires on every sentence each look perfect on one axis and
terrible on the other, and both mistakes have been made in this repository.
**Three measurements, always together:**

| | asks | how |
|---|---|---|
| **Precision** | does it stay quiet on correct text? | lint an edited-prose corpus; every report is a false positive |
| **Recall** | does it fire when it should? | inject the mistakes the rule targets at known offsets |
| **Per class** | which kinds of error does it reach at all? | a hand-built battery, compared against another checker |

```bash
cargo build --release --bin harper-cli --features de
# Precision — the corpus is edited prose, so the target is zero
./target/release/harper-cli lint --dialect de --only <Rule> --format compact \
    .archive/german-language/corpus-prose/*.md | wc -l
# Recall — see german/scripts/german_recall_check.py for what it injects
just language-recall german .archive/german-language/corpus-prose
# Precision on the committed sample, per rule
just language-lint-sources german
```

Only German has an injection set so far. A new language needs its own before
any recall claim means anything; copy the shape of `german_recall_check.py`,
which injects one real error class per row rather than random perturbations.

**Read the reports a change removes before believing the count.** Every useful
finding this repository has about German capitalization came from reading forty
lines of context, not from a total. Twelve reports that all turn out to be the
same construction are one bug, and the fix is usually to take something out of a
list rather than to add a guard.

**Never compare an `--only` count with a whole-group count.** They measure
different things: a lint group drops overlapping reports by priority, so
`GermanNounCapitalization` gives 2772 on its own and 2550 inside the curated
group on the same corpus. Mixing the two makes a pure refactor look like a
222-report regression. Pick one and use it for both sides of every comparison —
`--only` for a rule under development, the whole group for what a user sees.

**Diff sorted output.** Files are linted in parallel, so the report order varies
between runs of the same binary; an unsorted `diff` shows hundreds of moved
lines and no real change.

### Comparing against LanguageTool

LanguageTool is the practical oracle for the non-English languages: it is open,
it runs headless, and it names its rules, which a diff of two counts cannot.

```bash
docker start lt-bench        # port 8010; see german/README.md for the image
curl -s -X POST http://localhost:8010/v2/check \
     -d "language=de-DE" --data-urlencode "text@some-article.md"
```

To compare over a corpus: POST each file, chunk it at ~15 000 characters and add
the chunk offset back to every match, then align against
`harper-cli lint --dialect de --format json`, whose lints carry
`span.char_start` and `span.char_end`. Two spans that overlap are the same
finding.

**Two traps, both of which have already cost a wrong conclusion here:**

1. **Send the file exactly as Harper sees it.** Stripping markdown headings
   before the POST shifts every offset, and it also makes LanguageTool's
   `DE_CASE` fire three hundred times — the heading runs into the next sentence,
   so every sentence opener looks like a capitalized word mid-sentence. That is
   an artefact of the preprocessing, not a finding.
2. **Key the diff on `(file, line, column, flagged word)`, never on the whole
   message.** Harper's suggestion lists are not reproducible between runs:
   `fuzzy_match` caps its candidate set and which candidates survive depends on
   hash order. A line-based diff of two corpus runs of the *same binary* invents
   hundreds of differences.

**How to read the result.** Split the findings three ways:

- **both** — the shared floor, and usually the least interesting.
- **only Harper** — a precision problem. Group by our rule name, then look at
  the flagged words: they name the class. On German this pile was 2 000 reports
  and reading twenty of them showed it was bibliographies, not a dictionary gap.
- **only LanguageTool** — the missing-rule backlog. Group by *their* rule id,
  because the id says what kind of rule is missing, which a count never does.
  Ignore the raw total: a large share of it is their spell checker disagreeing
  with ours about proper names.

### The battery

For the per-class picture, write sentences rather than mining a corpus: edited
prose contains almost no grammar errors, so it can only ever measure precision.

A battery is N error classes × (5 wrong + 5 correct) in a TSV of
`class<TAB>err|ok<TAB>sentence`. Put one sentence per paragraph in a single
markdown file so one `harper-cli` invocation covers all of them — the German
dictionary costs about 1.5 s to load, so a sentence-per-process loop takes
minutes instead of seconds. Send the identical file to LanguageTool and map its
offsets back to sentences.

Score detection **and** false alarms together and report both; a class scored
without its correct sentences is not scored. And check *why* each hit fired: in
the first German run one apparent das/dass hit was a capitalization false
positive that happened to land on the right sentence, and it fired on the
correct sentence too.

German has a runner and its batteries checked in, and they are worth copying
rather than rewriting:

```bash
docker start lt-bench
cargo build --release --bin harper-cli --features de
python3 harper-core/src/language/german/scripts/german_battery.py \
    harper-core/src/language/german/tests/batteries/grammar.tsv
#   --per-sentence  to see what each tool said about each sentence
```

Two traps are baked into it, both of which cost a rewrite an hour:

* **Exclude typography by rule name, never by lint kind.** Filtering out the
  whole `Punctuation` kind to keep `CommaFixes` from scoring also removes
  `GermanSubordinateComma`, which is the comma class's own rule, and the class
  silently drops to zero. LanguageTool has the same shape: its comma rules are
  `issueType: typographical`.
* **`--format compact` prints basenames**, not the paths it was handed, so
  recover the sentence from `NNN.md`.

Keep an existing battery fixed once it has a recorded score — it is the only
comparable baseline. A new position or class gets a **new** file beside it, the
way `verb_final_agreement.tsv` sits next to `grammar.tsv`. A change that scores
the same on the old battery has not necessarily done nothing; it may have
improved something the old battery never contained.

### Errors from a treebank's gold annotations

The injection set and the batteries both contain what someone already thought
of. To find the classes nobody has, use a Universal Dependencies treebank as
the oracle: every token carries its lemma, case, number, person and head, so
one token can be swapped for **another attested form of the same lemma that
differs in exactly one feature** — *die Geräte sollen* → *soll*, *hat
verschoben* → *verschieben*, *mit dem* → *den*. Every variant is a real word
in the wrong place, and recall comes out per error class *and* per context
(which auxiliary, which preposition, subject before or after the verb), which
is where the gaps show.

```bash
python3 harper-core/src/language/german/scripts/german_treebank_injection.py \
    target/release/harper-cli out.json de_hdt-ud-test.conllu de_gsd-ud-test.conllu
python3 harper-core/src/language/german/scripts/german_treebank_injection.py \
    --score out.json verb_number      # recall per context for one class
```

Two cautions. **Read a sample of the variants before trusting a class**: a
swap can land on a correct form — a subjunctive when mood is not in the
features (*wir kämen*), either case behind a two-way preposition (*in die* /
*in der*) — and the class then looks worse than it is. And it measures recall
only: the same treebank's untouched sentences are the precision side, so lint
them before and after every change and read each new report. On German the
first run found two classes near zero (verb number with a noun subject, 0.6 %;
infinitive and participle swapped, 0.9 %), and the fixes for them found real
errors in the treebank's own news text.

When a class found this way gets a rule, it gets a battery file of its own as
well, per the paragraph above.

**The same annotations are a data oracle**, and for a language whose
dictionary lacks a feature, often the best one available. German noun gender
went from 68.5 % to 86.1 % of noun occurrences this way (see
`german/scripts/add_german_treebank_genders.py`). Three rules keep it honest:

* **Fill from the train splits, measure on the test splits.** Taking the
  data from the sentences the injection is scored on grades the work with
  its own answers.
* **Fill gaps; report contradictions.** Measure the oracle's agreement with
  what the dictionary already records first, then read every disagreement:
  the treebank is sometimes the wrong one (German *Inland*), and a word with
  two genders makes both look wrong.
* **Store the facts, not the treebank.** UD annotation is usually CC BY-SA, so
  the script reads it from wherever it is and writes only the flags.

Then re-run the precision side and read every new report. A feature that was
missing switches on checks that had nothing to say before, and the new reports
are the rules' blind spots rather than data errors.

## Before handing back

```bash
cargo test -p harper-core                      # English-only path
cargo test -p harper-core --features multilingual
cargo test -p harper-core --features all-languages --test language_conformance
just check-languages                            # feature consistency, per-language tests, coverage
cargo fmt
```

If you touched anything outside `language/`, also run `just check-english-parity`:
it runs the harper-core suite with no language feature and again with every
language, and fails if any shared test disagrees.

`just language-*` recipes cover dictionary iteration without recompiling; run
`just --list | grep language-` for the current set.

## Dictionary memory: what is "inflated", and why

A language's `dictionary.dict` is not what ends up in memory. Each line is a
headword plus affix flags (`Arzt/~~Mh0A`). At load time every flag is expanded
through the rules in `annotations.json`, and **every generated form is its own
dictionary entry**. German goes from about 245,000 lines to about 700,000 word
forms; English has about 55,000 lines.

The shared representation is expensive per form: a `WordMapEntry` is 168 bytes
(an 80-byte `CharString` and an 88-byte `DictWordMetadata`) in a hash table that
rounds up to a power of two, and `FstDictionary` keeps that beside an FST. That
was 193 MB while expanding and about 240 MB afterwards for German.

German therefore does not use it. `german/spell/compact_dict.rs` stores every
spelling in one `char` array, each *distinct* metadata value once (a few
thousand; `derived_from` is kept apart because it differs for every word), an
open-addressed table for lookups and an FST for fuzzy matching. It implements
`Dictionary` and answers like `FstDictionary`; a test compares every word and
its metadata with a one-shot expansion. The expansion is run in blocks, and a
form is handed over as soon as no later block can produce it, which gives the
same result as one map (a plain per-block merge does not: which headword wins
`derived_from` depends on the code path). The expansion itself is not
deterministic about the capitalisation of forms two headwords spell differently
(`-in` / `-In`), so that is the one thing the test lets differ.

Measured with `harper-cli lint --dialect de` on a tiny input (release build,
whole process, peak resident set): **637 MB before, about 190-220 MB now**;
English is 172 MB. Two further causes were not in the dictionary at all:

- `CompoundChecker::set_base_dictionary` called `HashSet::clear()` on a set of
  three casings of every word. `clear` keeps the allocation. It is now never
  built for the dictionary path (`CompoundChecker::for_dictionary`).
- `harper-cli lint` built the English curated config for any language, which
  loads the English dictionary (about 130 MB). Other languages now start from an
  empty config.

Consequences for you:

- Adding an affix flag that fans out into many forms still costs memory per form,
  but about a fifth of what it did. Check the count before and after
  (`report_german_dict_memory` in `german/tests/bench_german_dict.rs`).
- Measure the whole process, not just the dictionary: `ru_maxrss` of a child
  process, or `valgrind --tool=massif` to see who owns the peak. Two of the three
  causes above were found that way and not by reading the dictionary code.
- A language server loads the English dictionary for language detection as well,
  so its German footprint is the sum, not the figure above.
- Run `report_german_dict_memory` alone. Another test in the same binary warms the
  `LazyLock` and it then reports zero.
- The build is two passes (a parallel one that only finds the last block that
  produces each form, then the real one), so loading takes about a second longer
  than before and the checker is built after the dictionary, not beside it. Both
  are deliberate trades for the peak.

## Compounds in the word list

About a third of the capitalized headwords split into two other headwords, and it is
tempting to drop them: the compound checker accepts them anyway. Do not without measuring.
`german/scripts/prune_german_compounds.py --framework <harper-lang-test>` asks the checker
about each candidate (`harper-lang-test --dump-metadata`) and keeps whatever it reads
differently. Without `--apply` it only counts.

On the current dictionary 9,583 of 33,979 candidates (28%) read identically. The rest lose
their number (the checker drops it on purpose, see `head_noun_agreement`), their noun
reading or, now and then, their gender (`Kastration`). Removing the 9,583 kept the lint
output on the German samples identical and every test green, and took about 1% off the word
forms: no measurable change in startup or peak memory. Lower-case compounds, mostly prefix
verbs, fare worse (`--lowercase`: 1,830 of 55,019).

`dictionary.dict` is generated, so a prune is undone by the next regeneration unless the
builder runs it. To remove more, make the checker derive the missing properties; do not
loosen the comparison.
