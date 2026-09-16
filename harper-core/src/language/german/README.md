# German Language Support for Harper

This directory contains the German language implementation for Harper, implementing the `LanguageModule` trait.

## Dictionary Structure

German now uses a single unified dictionary system:

- **Annotated Dictionary** (`dictionary.dict` + `annotations.json`): Words with explicit POS metadata and comprehensive word coverage

This approach is consistent with other languages like Portuguese and Slovak, using only uncompressed dictionary files.

### What the dictionary costs

Both files are `include_str!`-ed into the binary, exactly as the English
dictionary is, so nothing is read from disk at runtime. Compiling German in grows
a release binary by roughly the on-disk size of `dictionary.dict`.

Everything after that is lazy. The word list is parsed and the affixes expanded
on the first German lookup and never before, so an English-only run of a binary
that has German compiled in pays nothing — same wall time, same peak RSS, with or
without `de`.

The German side is not cheap. Affix expansion yields close to an order of
magnitude more entries than English, and building them takes a couple of seconds
and several hundred megabytes on first use. That cost is paid once per process.

Two consequences worth keeping in mind when touching `german_dict.rs`:

- Entries are numerous enough that **a single stray copy of the dictionary costs
  hundreds of megabytes**. The accessors there are all `Arc` clones of one of
  three `LazyLock`s, and none of them builds anything. Keep it that way, and use
  `FstDictionary::as_mutable()` rather than converting, which copies.
- `harper-cli` must not force `FstDictionary::curated()` before dispatching, or a
  German lint loads the English dictionary it never consults.

Linting German is *faster per byte* than English, because the Brill tagger and
the neural chunker are skipped (see "Resolving part-of-speech ambiguity" below).

To measure any of this rather than trust a number written down here:

```bash
just language-coverage german     # entry count, coverage, affix efficiency
/usr/bin/time -f '%e s  %M KB peak' \
    target/release/harper-cli lint --dialect de --quiet <file>
```

`/usr/bin/time -f %M` reports *peak* RSS. For the steady-state figure read
`VmRSS` from `/proc/self/status`; the two differ by a wide margin here, so always
say which one a number is.

### Noun Capitalization

German noun capitalization works differently from English:

- **All nouns are capitalized** (not just proper nouns)
- Uses dictionary metadata to identify nouns
- Handles ambiguous words (e.g., "versehen" can be noun or verb)

**Implementation**: `german_noun_capitalization.rs`

### Compound Words

German supports compound nouns (e.g., "Donaudampfschifffahrtsgesellschaft").

**Rules**:
- Compound words are NOT added to `dictionary.dict`
- Generated automatically from base words using affix rules

### Word Formation Rules

The `annotations.json` file contains:

- **Affix Rules**: Generate inflected forms from base words
- **Properties**: Map POS flags to metadata (e.g., `N` → noun, `V` → verb)
- **Morphological Patterns**: German-specific word formation

#### Verb affixes build on the stem, whatever the entry looks like

The conjugation affixes — `d` (-te), `e` (-ten), `f` (-e), `h` (-st), `i` (-t),
`j` (-en), `c` (-enden) — attach to the verb **stem**: `lern` + `te`. But the
dictionary stores most verbs as the **infinitive** (`lernen`, `wandern`,
`sammeln`), and only a minority as the bare stem (`spiel/~~Vej`).

Each of those rules therefore carries several mutually exclusive replacements
keyed on the end of the base. The `t` endings — `d` (-te), `e` (-ten), `i` (-t) —
carry hunspell de_DE's full set, because German inserts an **epenthetic `e`**
between a stem ending in `d`, `t` or certain consonant clusters and the ending:

| base ends in | strip | example |
|---|---|---|
| `e[lr]n` | `n` | wandern → wander**te**, sammeln → sammel**te** |
| `[dtw]en` | `n` | arbeiten → arbeit**e**te, reden → red**e**te |
| `[^dimntw]en` | `en` | machen → mach**te** |
| `chnen` | `n` | rechnen → rechn**e**te |
| `[^aäehilmnoöuür][mn]en` | `n` | öffnen → öffn**e**te, atmen → atm**e**te |
| `[aäeilmnoöuür][mn]en` | `en` | lernen → lern**te**, entfernen → entfern**te** |
| `un` | `n` | tun → tut |
| anything not ending in `n` | — | the bare-stem entries |

The trick is that **how much is stripped decides whether the `e` appears**: leave
the infinitive's own `e` in place (arbeit-e + te) or take it with the ending
(mach + te). Without this the rules produced `arbeitte` and `errichtte`, and
declined participles such as `verheiratet` and `vergoldete` were reported as
misspellings.

The verb endings that do not start with `t` need none of it: `f` (-e) *is* the
epenthetic vowel, and `c` (-enden) and `j` (-en) attach after the whole `en` is
stripped. Those three keep the simpler four-shape table (`en`/`ern`/`eln`/bare
stem, plus the `[^erl]n`, `[^e]rn`, `[^e]ln` patterns that keep the bare-stem case
from overlapping — `Matcher` is fixed-length and end-anchored with no
alternation, so one pattern cannot express "ends in n but not in en/ern/eln").

`j` is the exception to the table: for `-ern`/`-eln` verbs the plural and the
infinitive *are* the base (`wir wandern`), so it puts the `n` back.

Before this, every affix simply appended to the entry, so the infinitive-shaped
entries — the large majority — generated `studierenten` and `lernente` while
`studierte` and `lernte` were reported as misspellings.

A verb reading (`V`) does **not** imply conjugation: the forms only exist if the
entry also carries `d`/`f`/`i`/`j`. Nor is `V` a reliable way to *find* the verbs
— it is missing from plenty of them (`promovieren/~~Nh`, `herrschen/~~XZ`) and
wrongly present on plenty of nouns.

`scripts/add_german_verb_conjugation_flags.py` therefore asks Hunspell instead:
an entry gains `dfij` only when the expanded form list accepts *every* form the
four flags would generate, matched case sensitively.

```bash
unmunch /usr/share/hunspell/de_DE.dic /usr/share/hunspell/de_DE.aff > forms.txt
scripts/add_german_verb_conjugation_flags.py --forms forms.txt --apply
```

It computes those forms by **reading the rules out of `annotations.json`** and
applying them, rather than from a second hand-written copy of German verb
morphology. Keep it that way. A paraphrase that missed the epenthetic `e` would
silently skip every verb with a `d`- or `t`-final stem — the script would report
no misses at all, because the forms it looked for were never the forms the rules
produce.

Two things the oracle alone does not settle:

- `j` (-en) regenerates an `-en` entry unchanged, so hunspell accepts it for
  every noun plural in the file. Only the `t` endings count as evidence of a
  verb; `f` and `j` are taken along afterwards.
- The form list carries lower-case *noun* forms too, because compounding needs
  them. "Konzern" would otherwise be conjugated on the strength of `konzert` and
  `konzerte`. A real verb form has no capitalized twin — at least one generated
  form must be absent in capitalized shape.

Re-running the script is idempotent. Its effect on `just language-coverage
german` is easy to misread: missing conjugation flags cost coverage against the
base list as well, because several lemmas are only reachable through them.

#### Where the `-ung` nouns come from

`Entscheidung` was missing from the dictionary, and so were `Bevölkerung`,
`Veröffentlichung` and several thousand more. Not a regression — no revision of
`dictionary.dict` ever had them, because igerman98 does not store them either.
It stores the **verb**, `entscheiden`, with the suffix flag `J`, and derives both
the noun and its plural from it.

What igerman98 *does* store capitalized is `Entscheidungs/hij`, and that is a
compounding stem carrying `NEEDAFFIX`: hunspell rejects it as a word on its own,
and it exists only to build `Entscheidungsträger`. Harper's import kept that stem
as an ordinary entry — which is why `entscheidungs` is in the dictionary and
`entscheidung` was not — and dropped the derivation.

Flags `7` (`-ung`) and `8` (`-ungen`) now mirror hunspell's `SFX J`. The suffix
attaches to the stem, so the infinitive ending comes off first, and the
conditions are all plain character classes that `Matcher` can express:

| base ends in | strip | add | example |
|---|---|---|---|
| `en` | `en` | `ung` | entscheiden → Entscheidung |
| `ern` | `n` | `ung` | ändern → Änderung |
| `eln` | `eln` | `lung` | sammeln → Sammlung |
| `[bgkpßsz]eln` | `n` | `ung` | wechseln → Wechselung |
| `el` | `el` | `lung` | handel → Handlung |
| anything but `n` | — | `ung` | zahl → Zahlung |

`scripts/mirror_hunspell_flag.py --from J --to 78` hands the pair to the verbs
igerman98 marks with `J`, verifying every generated form against the expanded
list first.

One trap in that script is worth repeating: `de_DE.aff` declares
`SET ISO8859-1`, but the `.dic` it ships next to is a symlink to the frami
variant, which is UTF-8. Decoding on the declaration silently mangles every
umlaut, and the script then reports no misses at all — because the entries never
matched in the first place.

#### Rules that do not fire

`k`, `l`, `m` and `n` (past participles) have conditions such as
`(be|er|ver|zer|ent|emp|miss)[^t]` and `ge[^t]`. `Matcher` has no alternation or
anchoring: it reads `(`, `b`, `e`, `|` … as literal characters and matches only
against the **end** of the word. Those four rules are consequently dead or
nonsense, and hardly any entry carries them. Participles are covered by explicit
dictionary entries instead (`gelernt`, `geschrieben`).

Do not assume a flag in `annotations.json` is productive. Count its users first:

```bash
just language-stats german
grep -c '^[^#]*/[^ #]*7' dictionary.dict   # entries carrying the -ung flag
```

#### Borrowing igerman98's flag membership

Mirroring a hunspell rule into `annotations.json` is only half the job: the flag
still has to reach the right entries. igerman98 already knows which words take
it, so `scripts/mirror_hunspell_flag.py` copies that membership across and then
*checks* the result — a Harper flag is added only when the expanded form list
accepts every form the rule would generate for that entry.

```bash
unmunch /usr/share/hunspell/de_DE.dic /usr/share/hunspell/de_DE.aff > forms.txt
scripts/mirror_hunspell_flag.py --forms forms.txt --from J --to 78 --apply
```

| hunspell flag | Harper flag | what it is |
|---|---|---|
| `J` | `7`, `8` | `-ung` nominalization and its plural |
| `U` | `9` | `un-` prefix, cross-product so the prefixed form still declines |
| `A` | `O Q R S T` | adjective declension, including on participles |
| `D` | `c` | present participle, declined |
| `C` | `U`, `W` | comparative and superlative, declined |

It also runs the other way. `--prune --to UW` **removes** a flag from every entry
whose generated forms hunspell rejects, which is how a flag that was handed out
too freely gets cleaned up:

```bash
scripts/mirror_hunspell_flag.py --forms forms.txt --to UW --prune --apply
scripts/mirror_hunspell_flag.py --forms forms.txt --from C --to UW --apply
```

That pair is worth understanding, because the first half is what makes the second
half affordable. Widening `U` and `W` from one form each to the full declined
paradigm fixed `westlichste` and `komplexesten` — and, applied to every entry
that happened to carry them, generated `aalartigere` and `aachtalster` by the
hundred thousand. Pruning first removes the flag from the place names and
non-gradable adjectives that should never have had it; only then is the wider
rule a net gain. Measure both directions, not just the corpus:

```bash
just language-coverage german     # reports Harper's expanded word count
# then compare that list against `forms.txt` to see what share it rejects
```

The script reads the rule out of `annotations.json` and applies it, so it cannot
drift from what Harper will actually generate. It handles prefix rules too: their
conditions describe the *start* of the word, not the end.

Two flags left this way still had to be extended first — `c` used to emit a
single `-enden` form, and now emits the whole declined participle. Extending an
existing flag beats claiming a new character: the namespace is nearly exhausted.

#### Affixes and properties share one namespace

`annotations.json` has two tables, `affixes` and `properties`, and a flag that
appears in **both** is applied as both. Most of the overlaps are deliberate and
say the same thing twice — `X` generates the `-e` plural and also marks the base
a plural noun. Several do not, and those are traps:

| flag | as an affix | as a property |
|---|---|---|
| `A` | `be-` prefix | adjective |
| `C` | `-keit` | conjunction |
| `D` | *(removed — see below)* | determiner |
| `F` | `-chen` | feminine noun |
| `I` | compound `-s` interfix | pronoun |

`D` is the reason `-ung` is not on `D`: handing it to the verbs turned every one
of them into an article, and the noun-phrase chunker then read half the corpus
as a determiner sequence. The `-ung` derivation lives on `7` and `8` instead, and
the `un-` prefix on `9` — digits, because they were the only characters free in
both tables. Digits were already the established escape hatch here: `4`, `5` and
`6` carry determiner, pronoun and conjunction for exactly the same reason. Only
`0` is left, so prefer extending an existing flag to claiming it.

The `A`, `C` and `F` *affixes* have been deleted. `F` was the expensive one: it
sat on every feminine noun as a property, so the affix was appending `-chen` to
all of them (`aufklärungchen`, `arzneichen`). The properties stay; only the affix
definitions are gone. Check any new flag against both tables:

```bash
python3 -c "import json; d=json.load(open('annotations.json')); \
  print(sorted(set(d['affixes']) & set(d['properties'])))"
```

#### The affixes over-generate

A large share of the expanded word list is strings no German dictionary accepts:
`N` appends `-es` to every noun it is on, `Y` tries both `-n` and `-en` so one is
always wrong, `a` is meant to be the umlaut plural but has no umlaut in it
(`mann` → `manner`). Measure it rather than guessing — expand hunspell, expand
Harper, and subtract:

```bash
unmunch /usr/share/hunspell/de_DE.dic /usr/share/hunspell/de_DE.aff > forms.txt
just language-coverage german     # reports Harper's expanded word count
```

`h` is in the same bind for a different reason. It has no property twin, but it
is both the `-st` verb affix (so every noun carrying it also generates
`arzneist`) and the flag `CompoundChecker` reads as "may form compounds".
Dropping it from the nouns would silently change what decomposes.

Pruning is therefore not simply a matter of dropping flags: `N`, `M`, `X`, `Y`
and `a` are the colliding letters above, and `h` is load-bearing elsewhere.
Separating the namespaces has to come first.

## The linters

`module.rs` registers the Rust linters; the Weir rules are discovered from
`linting/weir_rules/de/*.weir` by `build.rs`, which also generates one test per
rule from its own `test` and `allows` lines. **Adding a `.weir` file is the whole
edit** — no registration, no test module.

Two of the Rust linters cover mix-ups the spell checker structurally cannot see,
because the compound splitter reads the wrong spelling as a legal compound:

| Linter | Catches | Why spell check misses it |
|---|---|---|
| `german_wider_wieder.rs` | `wiederspiegeln` → `widerspiegeln`, `widerholen` → `wiederholen` | splits as `wieder` + `spiegeln`; both are words |
| `german_absolute_superlative.rs` | `einzigste` → `einzige` | the affix rules generate the superlative productively |

Both consult closed stem lists only. `wider`/`wieder` is genuinely ambiguous for
most stems (`widerhallen` and `wiederholen` are both correct), so anything not on
a list is left alone.

The Weir rules cover fixed misspellings in the same category — errors that stay
invisible to the spell checker because the wrong spelling decomposes into real
words. They fall into three groups:

- **Closed up that should be split**: `garnicht`, `garkein`, `aufjedenfall`,
  `desweiteren`, `wieviel` (two words since the 1996 reform).
- **Split that should be closed up**: `irgend etwas`, `irgend jemand`,
  `irgend wann`, `irgend wo`.
- **Single misspellings**: `Standart` (reads as `Stand` + `Art`),
  `Vorraussetzung` (`vor` + `raus` + `setzung`), `Diskusion` (`Diskus` + `Ion`),
  `Addresse`, `nähmlich`, `wiederrum`, `Vorraus`.

Two grammar rules sit alongside them: `VergleichAls.weir` rewrites `wie` to `als`
after a comparative, and `SeidSeit.weir` corrects the verb `seid` to the
preposition `seit` in front of a past or duration expression. The `seid`/`seit`
pair is only decidable in that one direction — in *"ihr seit Jahren bestehender
Betrieb"* the `seit` is correct — so the other direction is deliberately left
alone.

Every rule here is expected to be **silent on the archived corpus**. A firing
there is a false positive until shown otherwise; check before committing one:

```bash
just language-lint-sources german .archive/german-language/corpus
```

Two Weir traps worth knowing:

- Word matching is **case-insensitive**, so a rule for `des weiteren` also fires
  on the correct `des Weiteren`. Only match the unambiguous closed-up spelling.
- `MatchCase` copies the source token's case onto the whole replacement, so a
  one-token source such as `aufjedenfall` would produce `auf jeden fall`. Use
  `Exact` whenever the replacement contains a word that must stay capitalised.
- `becomes` is one string per rule, not per pattern, so a misspelling and its
  plural need two files — `Diskussion.weir` and `Diskussionen.weir`. Folding them
  into one alternation rewrites the plural to the singular.

### Tokens that are not words

`GermanSpellCheck` skips single letters, tokens containing a digit, Roman
numerals, and all-caps runs of up to five letters before it consults the
dictionary. Encyclopedic German is full of these — "Ludwig XIV.", "S. 11",
"ISBN", "der FC Bayern", "5 m" — and each one used to be reported, often with an
absurd suggestion (`II` → `in`). On the `.archive` corpus they were the single
largest class of false positive.

The five-letter cap is what keeps a genuinely misspelled word set in capitals
from being waved through.

A **capital inside** a short token marks it the same way: `gGmbH`, `UdSSR`,
`RoHS`, `VdS`, and the unit symbols `kV`, `dB`, `mA`, `CaO`. German orthography
has no word-internal capital, so declining to spell-check these costs nothing,
and each of them otherwise draws a suggestion list of pure noise. The length cap
here is six — past that, a stray capital is likelier a typo in a real compound
than an acronym.

## Development Guide

### Adding New Words

1. **Add to `dictionary.dict`**:
   ```
   Mondlandung/~~NF  # feminine noun
   schreiben/~~V     # verb
   ```

2. **Add properties to `annotations.json`** if needed:
   ```json
   "properties": {
     "NF": {"metadata": {"noun": {"gender": "FEMININE"}}}
   }
   ```

3. **Test with metadata tools**:
   ```bash
   just language-meta german "Mondlandung"
   just language-test german "die mondlandung ist wichtig"
   ```

### Testing

**Basic testing**:
```bash
just language-test german "der mond ist aufgegangen"
```

**Metadata inspection**:
```bash
just language-meta german "versehen"
just language-meta-text german "das war ein versehen"
```



### Debugging

**Common issues**:

1. **Word not recognized**: Add to `dictionary.dict`
2. **Wrong POS tag**: Fix flags in `dictionary.dict` or add properties
3. **False positives in capitalization**: Tag the word in `dictionary.dict` — see
   "Lexical classes" below. Do not add words to a Rust `const`.
4. **Missing inflected forms**: Add affix rules to `annotations.json`

### Lexical classes

`GermanNounCapitalization` must not "correct" words that are legitimately lower
case. Three such classes are **dictionary data**, carried by property flags:

| Flag | Class | Example entry |
|------|-------|---------------|
| `1` | Spelled-out cardinal numeral | `zwei/~~hJOQRSTUWq1` |
| `2` | Unit abbreviation | `kwh/~~2` |
| `3` | Lower-case Latin/Greek term | `facto/~~NhY3` |

To stop a word being flagged as a miscapitalized noun, add the appropriate flag
to its dictionary entry. `spell/lexical_classes.rs` reads these back into sets
once per process; no Rust change is needed.

Three lists remain in `german_noun_capitalization.rs` on purpose, because they
are closed grammatical classes the linter reasons over rather than vocabulary:

- `GERMAN_NON_NOUNS` — function words the dictionary actively mistags (a large
  fraction of them still carry a noun flag), so the dictionary cannot be the
  source of truth for them yet.
- `NOUN_PHRASE_LICENSORS` — the left-context test for "is this inside a noun
  phrase"; consulted only after the `is_preposition`/`is_determiner` metadata
  fast path fails.
- `SEPARABLE_VERB_PREFIXES` — a *prefix* match (`herausrückt`), not a membership
  test, so it cannot be a dictionary lookup.

**Note on flag characters**: many letters are simultaneously a property *and* an
affix rule, so adding a letter flag to a word can generate unintended forms
(tagging `mein` with the determiner flag `D` would also produce `meinung`). Only
digits and punctuation are free; new property flags should use digits.

## Comparison with English

| Feature | German | English |
|---------|--------|---------|
| **Dictionary** | Single annotated dictionary | Single dictionary |
| **POS Tagging** | Dictionary metadata | Brill tagger |
| **Noun Capitalization** | All nouns capitalized | Only proper nouns |
| **Compound Words** | Generated by rules | Explicit entries |
| **Irregular Forms** | Handled by annotations | Separate JSON files |

### Resolving part-of-speech ambiguity

Words that are a noun in one context and a verb or adjective in another are not
a German peculiarity — English has just as many (*fang*, *run*, *light*,
*present*). What differs is the machinery available to resolve them, and it
matters far more for German because `GermanNounCapitalization` has to act on the
answer for **every** noun, not just proper nouns.

The dictionary is equally ambiguous in both languages: `DictWordMetadata`
carries *every* reading a spelling has, and `is_noun()` / `is_verb()` mean "has
such a reading", never "is one here". English then narrows it down with two
**contextual** signals that `Document::parse` attaches to each token:

- `pos_tag` — a single best UPOS chosen in context by the trained Brill tagger
  (`harper-brill`). Falls back to `DictWordMetadata::infer_pos_tag`, which only
  answers when exactly one reading exists.
- `np_member` — noun-phrase membership from a neural chunker (`burn_chunker`).

English linters read those, and where they cannot, they simply *back off*:
`DictWordMetadata::is_likely_homograph` ("more than one part of speech") guards
`need_to_noun`, `oxford_comma`, `repeated_words` and others, which decline to
fire rather than guess.

**Neither signal is usable for German.** Both models are English-trained. On
German they produce almost all `None`, with the occasional outright error (`die`
is tagged `VERB`, as in English *to die*), and no German linter reads either.

They used to run anyway, because `Document::parse` was language-agnostic, and
they were the larger half of the cost of building a German document.
`Parser::is_english` is now the signal that turns them off: `PlainGerman`
answers `false`, and the Markdown and Org parsers the registry builds around it
inherit that. Skipping them roughly halves the cost of `Document::new` on German
prose, and `pos_tag` keeps whatever `annotations.json` supplies for the word
instead of being overwritten by an English guess.

German therefore recovers the structure itself, in
`german_noun_capitalization.rs`. It can afford to, because German noun phrases
are rigid where English ones are not: **determiner/preposition → attributive
adjectives → head noun**, with the adjectives inflected and the head
capitalized. `noun_phrase_roles` walks each sentence and labels every token
`Head`, `Modifier` or `Outside`:

```text
die   wesentliche   Frage        der   große      schöne     hund
^open ^Modifier     ^Head        ^open ^Modifier  ^Modifier  ^Head
```

Only a `Head` can be a miscapitalized noun. A `Modifier` is the attributive
adjective (*die **wesentliche** Frage*), and `Outside` is the verb reading
(*..., **fang** an*). The same word is flagged when it heads the phrase —
*das **wesentliche*** → *Wesentliche* — which is what keeps "das Wesentliche"
and "die wesentliche Frage" apart.

This replaced a one-token lookback ("is the word to my left an article?"), which
flagged every modifier and missed the head as soon as an adjective stood between
the two: in *der große schöne hund* it flagged `große` and never reached `hund`.

**Ending the phrase early is the expensive mistake.** Whatever token the walk
stops on becomes the head, so anything that interrupts a phrase before the noun
promotes the attributive adjective in front of it — and attributive adjectives
are far and away the largest source of false positives this rule has. Four
interruptions are stepped over rather than stopped on:

| In the text | Would otherwise stop at |
|---|---|
| *eine neue, radikalere Welle* | the comma, or `und`/`oder` |
| *der gerade oder **etwas** gekrümmte Griffel* | the degree word |
| *das beginnende **19.** Jahrhundert* | the numeral |
| *eine eigene **„**Baumnorm“*, *die deutsche **(**Wieder-)Besiedlung* | the quote or bracket |

A capital letter also outranks every part-of-speech reading on the token, which
it did not before: the dictionary hands out spurious adverb and verb readings
freely (`Band` is tagged an adverb), and rejecting the head on one of those was
enough to hand the role to the adjective before it.

Outside head position, two morphological shapes are rejected outright, because
neither is ever a noun and both arrive carrying a spurious noun reading:
declined adjectives (*britische* → *britisch*) and present participles
(*liegend* → *liegen*). Both are recognized from the stem the dictionary already
knows, not from a suffix table — which is what keeps *Abend*, *Jugend* and
*Tugend* out of the participle case. In head position the very same forms are
genuine nominalizations (*auf das wesentliche*, *nur für deutsche*) and stay
flagged, so the test is gated on the role.

`continues_noun_phrase` reads only the metadata already on the token. Do not add
a `Dictionary::get_word_metadata` call there — see the note in
`../AGENTS.md` about `CompoundAwareDictionary`'s global mutex.

### Auditing capitalization false positives

Edited German prose should produce essentially **zero** `GermanNounCapitalization`
lints. The archived Wikipedia articles under `.archive/german-language/` are the
working corpus for this; a lint there is a bug until proven otherwise.

```bash
just language-lint-sources german .archive/german-language/test-sources
```

To decide whether a flagged word is a real error, use `aspell` as an oracle. A
German spelling dictionary lists nouns **only** capitalized, so it accepts a
lower-case spelling exactly when the word is legitimately lower case:

```bash
$ echo hund        | aspell -d de -a --encoding=utf-8   # & hund … Hund  -> real error
$ echo wesentliche | aspell -d de -a --encoding=utf-8   # + wesentlich   -> false positive
```

`hunspell` works too, but only through `iconv` — the shipped `de_DE` dictionary
is ISO-8859-1 and silently drops umlauts on UTF-8 input, so every word containing
`ä ö ü ß` comes back "misspelled" unless you convert first:

```bash
$ echo lernente | iconv -f utf-8 -t iso-8859-1 | hunspell -d de_DE -a
```

#### The expanded form list is the better oracle

Asking `aspell` or `hunspell` one word at a time answers "is this a word". For
anything that needs a *set* — which forms are missing, which entries deserve a
flag — expand the Hunspell dictionary once instead and compare against the
result:

```bash
unmunch /usr/share/hunspell/de_DE.dic /usr/share/hunspell/de_DE.aff > forms.txt
```

This is the reference that matters. The list committed next to this README,
`german_dictionary.dict.gz`, is the Hunspell **base** list: lemmas, no inflected
forms. `just language-coverage german` measures against it, which is why it
cannot see a broken conjugation rule — every lemma still resolves while every
form built from it is wrong. The `unmunch` output contains the forms themselves
and catches exactly that class.

Match it **case sensitively**. German verbs are lower case and nouns are
capitalized, so here the casing *is* the part of speech. That is what stops
`bären` from being conjugated as though it were a verb: the form `bärte` would
have to exist, and the list has only `Bärte`, the plural of `Bart`. Harper's own
dictionary cannot make this distinction — see the note on lower-cased entries
under [Known Gaps](#known-gaps).

Strip the annotation lines `unmunch` emits alongside the words before using it:

```bash
grep -vE '[|/]' forms.txt | grep -E '^[A-Za-zÄÖÜäöüß-]+$' | sort -u > oracle.txt
```

`aspell` is a spell checker only — it has no grammar rules at all (its "modes"
are input filters for markdown, HTML, TeX and so on). For a grammar-aware
oracle, use **LanguageTool**, which is available as a local container:

```bash
docker run -d --name lt-de -p 8010:8010 -e Java_Xmx=4g erikvl87/languagetool:latest
```

Its German rule set is more than an order of magnitude larger than Harper's, so
on German prose it is effectively a superset and a good arbiter: a Harper lint
that no LanguageTool match overlaps is a false positive.

**Read its rules for the map, not for the content.** LanguageTool is LGPL and
Harper is Apache-2.0, so its `grammar.xml`, `replace.txt` and the rest cannot be
copied or transcribed into this directory — that would make Harper's German
rules a derivative of LGPL data. What its rule set is legitimately good for is
telling you *which error categories are worth having*: casing,
Getrennt-/Zusammenschreibung, easily confused words, comma placement, typography.
Pick a category, then write the rule here from the German grammar rather than
from theirs, and verify it against the corpus.

The scratch tooling for this lives in `.archive/german-language/scripts/`
(untracked):

```bash
build_german_corpus.py --count 400        # fresh Wikipedia prose via the API
compare_with_languagetool.py <corpus> --rule GermanNounCapitalization \
    --json-out suspects.json              # triage Harper's lints
derive_pos_fixes.py suspects.json         # -> scripts/german_pos_fixes.tsv
scripts/fix_german_pos_flags.py --apply   # append the missing readings
```

`scripts/german_pos_fixes.tsv` **is** tracked — it is the record of which words
LanguageTool vouched for and what reading each one was missing, so the
dictionary change stays reproducible.

The audit that motivated the current rules found the large majority of flagged
words accepted by `aspell`, in three recurring shapes:

1. **Suspended hyphenation** — *"auf welt-, volks-, stadt- und
   hauswirtschaftlicher Ebene"*, *"Konfliktverhütung und -lösung"*. Handled by
   `is_hyphen_compound_fragment`.
2. **Foreign-language glosses** — *"englisch economy, französisch économie"*,
   *"althochdeutsch reht, recht, rehd"*. Handled by `follows_language_gloss`
   with `LANGUAGE_GLOSS_MARKERS`.
3. **Entries with a corpus-mined noun reading and nothing else**, which the
   linter must treat as unambiguous nouns. Fixed in the dictionary by
   `scripts/fix_german_pos_flags.py`, whose additive pass *appends* the missing
   adjective/verb/adverb flag rather than replacing the entry — the word becomes
   a homograph and the noun-phrase chunker decides per occurrence.

## Implementation Notes

- Uses a single annotated dictionary for both word coverage and metadata
- Lookup speed is O(1) for most operations due to FST structure
- Dictionary construction is lazy and happens once per process

**No statistics in this file.** Entry counts, coverage percentages, lint totals
and timings all change the moment someone improves the dictionary, and a stale
number here is worse than no number. Record *how to measure* instead —
`just language-coverage german`, `just language-lint-sources german`,
`/usr/bin/time` — and let the tooling report the current value.

## Known Gaps

- **Vocabulary holes**: words are still missing outright — check with
  `just language-lint-sources german .archive/german-language/corpus`, and read
  the result against the expanded hunspell list rather than by eye.
  `scripts/add_german_missing_verbs.py` closes the verb side of this; nouns have
  no equivalent yet.

  Do **not** size this gap by diffing the expanded hunspell list against
  Harper's. That comparison says hundreds of thousands of words are missing and
  it is wrong: Harper resolves compounds at lookup time, so a word absent from
  the base expansion is usually still accepted. Only the corpus measures what a
  reader would actually see.
- **Over-permissive compound splitting**: the splitter accepts any chain of
  dictionary words, so misspellings that happen to decompose survive (`Standart`
  = `Stand` + `Art`, `Diskusion` = `Diskus` + `Ion`). The Weir rules patch the
  frequent cases one at a time. Note there are *two* splitters —
  `CompoundChecker`, which `CompoundAwareDictionary` consults on every lookup
  miss, and `GermanSpellCheck::try_compound_word_check`. The linter's copy is
  unreachable in the normal pipeline, because the dictionary has already said
  yes; the two share `MIN_COMPOUND_PART_LEN` so they cannot drift apart on what
  an element is.

  **A minimum length is not the fix.** Raising it from three to four was tried
  and reverted: it catches `Diskusion` and `Vorraussetzung`, and costs several
  hundred false positives on edited prose, because German builds just as freely
  on short *prefixes* (`vor`, `aus`, `auf`, `neu`, `süd`) as on short nouns. Half
  the new false positives were not explained by a short leading element at all.
  The elements have to be typed — prefix vs. noun — before any threshold helps.
- **Lower-case compounds are not caught**: `lernente` is wrong and `Lernente` is
  a (strange but well-formed) compound noun, and Hunspell draws exactly that
  line. Harper cannot, for two compounding reasons, and an attempt to add the
  rule was reverted after it produced false positives across the whole corpus and
  essentially no true ones:
  - the head of the decomposition decides the word class, but the heads that come
    back are junk (`wiederholt` splits as `wie` + `derholt`), because any
    affix-generated string of three or more characters is a usable element;
  - a noun-only head cannot be told from a derivational suffix that is homographic
    with a noun, so `dauerhaft` reads as `Dauer` + `Haft`.

  Both would have to be fixed before the capitalization rule is worth revisiting.
- **The dictionary is lower-cased**: almost every entry starts with a small
  letter, where the Hunspell reference capitalizes a large fraction of them. In
  German the capital *is* the noun marker, so `get_correct_capitalization_of` —
  the mechanism English relies on — returns the wrong answer for every German
  noun, and `GermanNounCapitalization` has to reconstruct from context what the
  dictionary should have stored. Restoring the casing from the reference list is
  the single largest structural improvement available.
- **The noun flags overstate**: `N`, `M`, `X` and `Y` are the `-es`, `-er`, `-e`
  and `-en` suffix rules, and each carries `base_metadata: {"noun": {}}`. Any
  entry given one of them reads as a noun, which is how adjectives (`hellblau`)
  and finite verb forms (`zeichnet`, `portiert`) end up with noun readings. Every
  rule keyed on "is a noun" inherits the error.
- **Strong verbs**: `dfij` generates a weak preterite for every verb it is
  applied to, so `berufte` is accepted alongside `berief`. Over-generation, not a
  false positive.
- **Dialect support**: Austrian and Swiss variants are declared but barely
  populated.

## References

- **Hunspell source**: igerman98 dictionary (GPLv2/GPLv3)
- **Word list**: Expanded using Hunspell affix rules
- **Metadata format**: Harper-specific annotations system

For more details, see the main [Language Support README](../README.md).