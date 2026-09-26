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

`harper-core/src/language/german/scripts/add_german_verb_conjugation_flags.py` therefore asks Hunspell instead:
an entry gains `dfij` only when the expanded form list accepts *every* form the
four flags would generate, matched case sensitively.

```bash
unmunch /usr/share/hunspell/de_DE.dic /usr/share/hunspell/de_DE.aff > forms.txt
harper-core/src/language/german/scripts/add_german_verb_conjugation_flags.py --forms forms.txt --apply
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

`harper-core/src/language/german/scripts/mirror_hunspell_flag.py --from J --to 78` hands the pair to the verbs
igerman98 marks with `J`, verifying every generated form against the expanded
list first.

One trap in that script is worth repeating: `de_DE.aff` declares
`SET ISO8859-1`, but the `.dic` it ships next to is a symlink to the frami
variant, which is UTF-8. Decoding on the declaration silently mangles every
umlaut, and the script then reports no misses at all — because the entries never
matched in the first place.

#### Strong verbs need a second entry, not a cleverer rule

Every conjugation flag above builds on the infinitive, and for a strong verb
that is a dead end: `ziehen` gives `zog`, not `ziehte`. The flags are withheld
from those verbs on purpose — `harper-core/src/language/german/scripts/add_german_verb_conjugation_flags.py`
requires hunspell to accept *every* generated form, and `verbietete` is not a
word — so a strong verb has a present tense and nothing else. `stattfanden`,
`ausschieden`, `überließen` and `vorhielten` were misspellings.

igerman98 does not try to derive them either. It lists the **preterite stem** as
a headword of its own (`zog/VZ`, `schrieb/VZ`) and the participle as a third
(`gezogen/AU`), and inflects the stem with `SFX Z`. Flag `s` mirrors that rule:

| stem ends in | add | example |
|---|---|---|
| `[^hßsz]` | `st` | zog → zog**st** |
| `[dfkstz]`, `ch`, `[au]ß`, `ieß`, `[io]ss` | `est` | hielt → hielt**est** |
| `[^dt]` | `t` | zog → zog**t** |
| `[dt]` | `et` | hielt → hielt**et** |
| `e` | `n` | abspräche → abspräche**n** |
| `[^e]`, `ie` | `en` | zog → zog**en** |

`harper-core/src/language/german/scripts/mirror_hunspell_flag.py --from Z --to s` puts it on the same entries
igerman98 does. `s` was the **last character free in both namespaces** — see the
collision table below; the next flag needs one of the digits back.

Two things this pass turned up that any further POS work will hit again:

- Nearly every one of those stems was mined as a **noun** (`zog/~~NhYr`), so the
  personal forms "appear to be a noun" mid-sentence. Removing the `N` property is
  not enough — the noun-plural affixes `X`/`Y` carry a plural noun reading of
  their own, and `zogt` stayed a noun until they came off too.
- The flag on `zog` does not reach `zogt`, because `zogt/~~NhYG` is an entry in
  its own right. `harper-core/src/language/german/scripts/fix_german_pos_flags.py` therefore expands the `s` rule
  itself and retags every form it produces, not just the stems carrying the flag.

And one thing that has to be left alone. German capitalizes its nouns, so a
lower-case entry with a noun reading looks like a mining error — but a slice of
`dictionary.dict` stores capitalized nouns in lower case and relies on the
case-insensitive lookup. There is no `Maß` entry, only `maß/~~NXh0`, and `maß` is
*also* the preterite of `messen`. Stripping its plural affix deletes `Maße`. The
pass needs the expanded form list to tell the two apart:

```bash
harper-core/src/language/german/scripts/fix_german_pos_flags.py --forms forms.txt --apply
```

Without `--forms` it skips the preterite pass rather than guessing.

#### What the dictionary costs, and what shrinking it buys

`harper-core/tests/bench_german_dict.rs` reports it. Run that one test alone —
another test in the same binary warms the `LazyLock` and it then reports zero:

```bash
cargo test -p harper-core --features multilingual \
    --test bench_german_dict report_german_dict_memory -- --nocapture
```

Measured before and after this branch removed 173793 generated forms, one build
per process:

| words | build | resident | peak |
|---|---|---|---|
| 790823 | 1.50 s | +277 MB | 473 MB |
| 617030 | 1.24 s | +249 MB | 422 MB |

A 22% smaller word list buys 10% less memory. The footprint is **sub-linear in
the vocabulary** — roughly 180 MB of it does not depend on the word count at
all, which is the shape issue #3725 describes: the FST is small and the cost is
what `FstDictionary` materializes alongside it for fuzzy matching. Shrinking the
word list is worth doing and is not the lever. A language more inflected than
German will not escape this by having fewer entries.

Watch the gap between peak and resident too, 422 MB against 249 MB: a language
server pays the peak at startup, not the steady state.

For speed the picture is the other way round. On the same 3.3 MB of prose,
`harper-cli lint` takes **26.9 s in German against 52.0 s in English** — German
skips the English Brill tagger and the neural chunker, which `Plain<Name>`
declines by overriding `Parser::is_english`. The language module is not the slow
path; the dictionary is the expensive part, and it is a fixed cost paid once.

#### The corpus decides which bugs you can see

The archived corpus is 264 random German Wikipedia articles, which in practice
means biographies, places and events. It is good at one thing — proper names
neither Harper nor igerman98 knows, which is a coverage question and a dull
one — and it barely exercises the grammar, because its sentences are short and
appositive.

```bash
harper-core/src/language/german/scripts/fetch_german_corpus.py .archive/german-language/corpus-prose
```

fetches the other kind of article: grammar, law, philosophy, mathematics,
medicine, economics. Abstract topics are written in long sentences with
subordinate clauses, nominalizations, participial attributes and passive voice,
and they name almost nobody. 101 whole articles, and
`GermanNounCapitalization` went from 144 lints on the old corpus to 1574 on the
new one — eleven times the density on twice the text. None of those bugs were
new; there had simply been no text in which to see them.

Keep both. The Wikipedia corpus is the regression test for everything the
dictionary covers; the prose corpus is where the rules get tested.

#### One flag cannot be two plurals

`Y` was documented as "Noun plural -n/-en" and meant it literally: every entry
carrying it got **both** `word + "n"` and `word + "en"`. A German noun takes one
or the other, so for nearly every one of the hundred thousand entries that
carried it, one of the two was not a word — and many take neither, because the
plural umlauts (`Arzt` → `Ärzte`), doubles an `s` (`Ergebnis` → `Ergebnisse`) or
does not exist (`Chemie`).

Against the unmunched igerman98 list, of the entries it can judge: 29065 take
only `-n`, 23196 only `-en`, 31 both, 48550 neither.

```bash
harper-core/src/language/german/scripts/split_german_plural_n.py --forms forms.txt --apply
```

`-n` moved to `E` and `Y` narrowed to `-en`, and each entry got whichever
igerman98 says it takes. About 148000 generated non-words fewer, and typo
detection went from 91% to 92% — this was the largest single source of junk in
the dictionary.

Two things to copy when splitting any other flag this way:

- **A flag that is also a property has to be split in both tables.** `Y` carries
  a plural-noun reading as well as generating forms, and
  `GermanNounCapitalization` leans on it: for a word ending in `-e` it treats a
  noun reading as real only when the entry carries gender or number, which is
  what keeps `für deutsche` flagged and `die festigende Wirkung` quiet. `E` was
  an affix only at first and four tests went red.
- **An entry igerman98 does not list is not judged.** It keeps both forms, so
  the compounds igerman98 composes rather than lists
  (`skalierungstabelle`) are untouched.

The cost is words that were only ever spelled correctly by accident.
`Programmen` and `Subjunktionen` are real, and Harper accepted them because the
compound checker could reach them through forms that were junk. Neither
`Programm` nor `Subjunktion` is an entry; importing the headword is the fix, not
keeping the junk.

#### Gender is wrong often enough to be unusable

Of the 4000 most frequent nouns of the prose corpus, roughly one entry in
fourteen that carries a gender carries the wrong one, and the mistakes are
systematic. An `-er` read as an agent-noun suffix made `Leber`, `Mauer`,
`Dauer`, `Nummer`, `Ziffer`, `Metapher` and `Kammer` masculine, and `Fenster`,
`Gewitter`, `Kloster`, `Theater` and `Wetter` too. `Tier`, `Meer`, `Papier` and
`Heer` are neuter and were masculine; `Raum` and `Irrtum` are masculine and were
neuter; `Bombe` and `Breite` carried masculine *and* neuter at once.

That is why `german_preposition_case.rs` narrows a determiner by number only.

**The oracle is the text, not another dictionary.** A German article names the
gender of the noun it introduces, and a few article forms do so with no
competing reading at all:

| cue | says |
|---|---|
| `eine`, `einer` | feminine |
| `einen` | masculine |
| `das` | neuter |
| `dem`, `des`, `einem`, `eines`, `diesem`, `dieses`, `keinem`, `keines`, `meinem`, `meines`, `seinem`, `seines`, `ihrem`, `ihres`, `jedem`, `jedes` | masculine or neuter, never feminine |

Everything else is ambiguous and stays out. `dieser` is nominative masculine
*and* dative feminine; `keine` is feminine singular *and* plural; `keinen` is
accusative singular *and* dative plural. Including `dieser` alone was enough to
make *Zeit* come out masculine, on 129 votes.

Two things in running text look like the pattern and are not, and both were
found by reading the disagreements rather than by thinking about it:

- a **hyphenated compound** — *das Kaiser-Wilhelm-Denkmal* votes for `Kaiser`;
- an **indeclinable attributive adjective** — *das Londoner Abkommen* votes for
  `Londoner`, and *der Schweizer Musik* makes `Schweizer` a masculine noun.

```bash
harper-core/src/language/german/scripts/audit_german_gender.py --corpus .archive/german-language/corpus-prose [--apply]
```

With both guards, the corpus agrees with 471 entries and contradicts 9, and
every contradiction was a genuine mistake. Checked the other way round — against
the derivational suffixes, which are an independent source — the corpus is right
99.7 % of the time. `tests/noun_gender_test.rs` holds the corrections.

**Filling the gaps.** The same script writes a gender where an entry has none,
from two sources. The corpus settles what it sees often enough; for the rest, a
few derivational suffixes settle it on their own, and those were measured
against the corpus rather than assumed:

| | |
|---|---|
| used | `-ung` 99.5 %, `-tion`/`-sion`/`-ie`/`-ismus`/`-nis`/`-ment` 100 %, `-chen` 92 %, `-um` 90 % |
| **not** used | `-e` 65 %, `-er` 61 %, `-el` 45 % |

The second row is the rule that caused the damage in the first place.

Most of the corpus evidence is `dem`/`des`/`einem`, which says only *not
feminine*. That is recorded as such rather than guessed further: a specific
gender needs that many votes of its own, because one stray `das` outvoting
three `dem`s made *Nutzer* neuter. A set of two genders still rules out a third
of the possibilities, which is what an intersection needs.

igerman98 is used to contradict, never to decide. A headword with the genitive
`-es` flag `T` is never feminine and one deriving an `-in` form is masculine,
which is sharp for `-er` nouns and noisy elsewhere — it mislabels `-ismus`,
whose genitive is uninflected, and umlaut-plural compound elements.

**A compound takes the gender of its head.** `CompoundChecker::get_compound_metadata`
already knew that a *Determinativkompositum* is right-headed and already found
the head for its adjective test; it now copies the head's gender onto the
compound. No dictionary lines, and it reaches the compounds that have no entry
at all — which is most of them, since German builds them faster than any word
list records them. The **number** is deliberately not inherited: *Schlüssel* is
a singular but *Hausschlüsseln* is a dative plural, and only the compound's own
ending says which.

Over the 4000 most frequent nouns of the prose corpus, gender coverage went
from 28.7 % to 53.7 %.

#### A noun that cannot form its plural had no number at all

A noun gets its number from the plural affix it carries. A noun whose plural
this dictionary cannot build carries none — `bruder`, `vater`, and 28266
others — so it said nothing, and *mit den Bruder* passed because the dative
plural reading of `den` stood unchallenged.

The `A` flag, previously an unused alias of the adjective property `J`, now
marks a singular noun. The flag namespace is full at 63 of 63, so a new flag has
to come from a retired one; `o` and `y` are the two still free.

```bash
harper-core/src/language/german/scripts/mark_german_singular_nouns.py --hunspell <dir>/de_utf [--apply]
```

The oracle is igerman98 through `hunspell -m`: a word it analyses as `st:<the
word itself>` with no plural flag is a base form, and a noun's base form is a
singular. Three classes are held back even so, and the third was found by
measurement rather than thought:

| held back | because | example |
|---|---|---|
| `-er`, `-el`, `-en` | spelled alike in both numbers | *der Lehrer*, *die Lehrer* |
| pluralia tantum | nothing in the morphology says so, so they are listed | *Eltern*, *Masern*, *Jeans* |
| `-a`, `-i` | a Latin or Greek plural, which hunspell reads as a base form because that is what it is | *Korpora*, *Charakteristika*, *Termini*, *Visa* |

Skipping the third cost the singulars that end the same way — *Kamera*, *Pizza*,
*Oma* keep no number — and reporting *zu den Korpora* six times was the
alternative.

Two other rules were leaning on the old silence and had to be told what they
actually meant:

- `GermanNounCapitalization` accepted a bare `-e` word as a noun when the entry
  carried **any** agreement feature. Once a bare singular counted, `file`,
  `single`, `hardware`, `grace` and `zuhause` all passed. It now asks for gender
  or a plural, which is the question it was reaching for.
- `suffixed_element_set` reads the feminine flag to decide that an element takes
  only the `-s-` interfix. Giving 1641 `-ung`/`-ion`/`-keit` nouns their correct
  gender therefore made `neurowissenschaftliche` and `konformationelle`
  misspellings. A derivational suffix is not a compound seam, so an element may
  now be followed directly by `-lich`, `-ell`, `-al` and the rest of that closed
  list. `Nationaal` and `Attraktivitätbeurteilungsskala` stay caught, which is
  the half of the change that was right.

#### A plural flag said the opposite of what it meant

`X`, `Y`, `a`, `b` and `E` are each an affix *and* a property, and the two said
opposite things. The affix marks the form it **builds**: `Frau` plus `Y` gives
`Frauen`, a plural. The property marked the entry it **sits on**, so `Frau` was
a plural too — and so was every other singular noun that can form one. **67268
of 67269.**

That is not a cosmetic mislabel. `den` is accusative masculine singular *or*
dative plural, and only the noun can say which. A noun claiming to be plural
keeps the dative plural reading alive, which is exactly what made
*"mit **den** Freund"* look acceptable. The whole class of case errors that only
the noun reveals was invisible because of it.

The number now lives on the affix alone, and four of the five mark their base
singular in return. `E` does not: it is carried both by a singular whose plural
is `-n` (`Diagnose` → `Diagnosen`) and by a form that is already plural and only
adds the dative `-n` (`die Lehrer` → `den Lehrern`, `die Befunde` → `den
Befunden`). For the `-er` and `-el` nouns the base really is both numbers at
once, so `E` marks its base `["Singular", "Plural"]` — a set that constrains
nothing, which is the honest answer, and which still tells
`GermanNounCapitalization` that a bare `-e` word is a noun rather than a verb
form.

Measured against the igerman98 hunspell dictionary as an oracle, over the 4000
most frequent nouns of the prose corpus:

| | before | after |
|---|---|---|
| number agrees with the oracle | 553 | **1670** |
| base form wrongly marked plural | 1174 | **56** |
| plural form wrongly marked singular | 0 | **1** |

The oracle needs care. `de_DE.aff` on this machine declares `SET ISO8859-1`
while `de_DE.dic` beside it is UTF-8, so every word with an umlaut silently
fails to analyse — half the German language. Copy the `.aff`, rewrite that one
line to `SET UTF-8`, and leave the `.dic` alone.

The one remaining conflict is `Fischen`, and it points at the next gap: the
dative plural of an `-e` plural (`Fisch` → `Fische` → `Fischen`) is not
generated at all. Where it exists it is an accident of a noun carrying `Y` as
well, and where it does not, the only entry is a lower-case compound element.

#### Words that are not nouns, tagged as nouns

German capitalizes its nouns, so a lower-case entry whose **capitalized** form
igerman98 does not list is not a noun, whatever corpus mining tagged it.
`allenfalls`, `gleichwohl`, `wenngleich`, `mithin`, `desto`, `derart`,
`hierdurch`, `diejenige` and tens of thousands of finite verb forms all carried
`~~NhY` or `~~NXh`, and `GermanNounCapitalization` flags anything with an
unambiguous noun reading. Taking the reading off the 35887 entries igerman98
can vouch for removed 1001 of the prose corpus's 1574 lints.

```bash
harper-core/src/language/german/scripts/strip_german_noun_readings.py --forms forms.txt --apply
```

Three things it has to get right, all learned the hard way:

- **`N` alone is not the noun reading.** The noun-plural affixes `X`, `Y`, `a`,
  `b` and `0` each carry a plural-noun reading of their own, so an entry stripped
  of `N` is still a noun.
- **Those affixes also generate forms**, and some are the only source of a real
  word. `bedachte/~~YsV` builds `bedachten` through `Y`. The script expands the
  whole dictionary before and after and puts an affix back rather than lose a
  form igerman98 lists — case-insensitively, because a lower-case entry is what
  makes a capitalized noun spell correctly.
- **An affix that has to stay can still change its mind about what it means.**
  `geh/~~Xh` was the only source of `gehe`, and `f` (verb present `-e`) builds
  exactly the same string, so the entry becomes `geh/~~fh` and `gehe` stops
  being a plural noun.

Two further guards keep the rule from eating things it should not. An entry with
an **adjective** reading is skipped entirely, and so is one that is a declined
form of an adjective the dictionary has: German nominalizes adjectives freely —
*das Gute*, *im Freien*, *für Deutsche* — and those really are nouns, but
igerman98's expanded list does not carry their capitalized spellings, so the
oracle would call every one of them "not a noun". `deutsche` and `wesentliche`
are the cases that caught this. And the word itself has to be in igerman98 in
*some* casing, or there is no basis for judging it at all: `university` is in the
German dictionary as a borrowing and in igerman98 in neither casing, and without
that guard the rule reads "never capitalized, therefore not a noun".

What it does not reach is a word the dictionary never lists. `verbleibt` is
spelled correctly only because the compound checker takes it apart, and
`get_compound_metadata` calls what it cannot classify a noun. That is a
`compound_checker.rs` question, not a data one.

#### A missing derivation costs more than a missing word

German derives the feminine personal noun from the masculine one with `-in`
(*Movierung*), plural `-innen`. The rule is fully productive and it was absent
from the model, so `Sprecherin`, `Ministerin`, `Politikerin`, `Regisseurin` and
`Professorin` were all reported as misspellings while `sprecher`, `minister`,
`politiker`, `regisseur` and `professor` sat in the dictionary.

It is hunspell de_DE's `SFX F`, and it is split across two Harper flags the way
`-ung`/`-ungen` is split across `7` and `8`: `K` builds the singular and `L` the
plural, so the plural forms carry a plural reading of their own instead of
borrowing the singular's.

```bash
harper-core/src/language/german/scripts/mirror_hunspell_flag.py --forms forms.txt --from F --to KL --apply
```

4548 entries took the flags and two were refused, because a flag is added only
when the expanded hunspell list accepts every form it would generate. Both flags
also mark the derivation **feminine**, which the masculine entry cannot: `näher`
carries no gender at all.

What this does *not* reach is a masculine noun that is not an entry. `Nachfolger`
is spelled correctly only because the compound checker takes it apart, and a flag
has nothing to attach to there, so `Nachfolgerin` is still missing. Importing the
headword is the lever for that — see the next section, and its cost.

The interior-capital spelling (`LehrerIn`, `LehrerInnen`) comes along, because
igerman98 lists it. Measured on all three axes it paid for itself: 71 corpus
false positives gone, typo detection up rather than down, injected-error recall
unchanged, and no new dictionary entries at all — 7833 more surface forms out of
the same 234938 lines.

#### The genitive of a name, and why it is capitalized only

German puts a bare `-s` on a proper name to form the genitive — *Rembrandts
Werke*, *Orwells Roman*, *Maximilians Nachfolge*. Harper had the names and not
the genitives. The rule is hunspell de_DE's `SFX S`, and it lives on `H`, not on
`b`: `b` is the same surface rule but declares both the result and the base a
**plural** noun, which is true for `Autos` and wrong for `Maximilians`.

```bash
harper-core/src/language/german/scripts/mirror_hunspell_flag.py --forms forms.txt --from S --to H \
    --only-capitalized --apply
```

`--only-capitalized` is the whole reason this is affordable, and it is worth
being precise about the cost, because the unrestricted version looked better on
the corpus:

| | entries | corpus lints | typo detection |
|---|---|---|---|
| without `H` | — | 12543 | 31575 |
| `H` on everything hunspell marks | 45545 | 12259 | 31391 |
| `H` on capitalized entries only | 6749 | 12448 | 31574 |

The unrestricted version removes three times as many false positives and gives
up 184 detections to do it — and they are not obscure. `anderen -> annderen`,
`jeweils -> jeweills`, `Anfang -> Annfang`, `danach -> daanach`: typos of the
most frequent words in the corpus. Every new surface form is also a compound
element, so 45545 of them reopen that many decompositions. The lower-case
entries are ordinary common nouns, `0` (`-es`) and `b` (`-s` plural) already
serve them, and dropping them costs a third of the gain and one detection.

The limit is the same one the feminine derivation has: a name that is not an
entry has nothing for a flag to attach to. `Österreich`, `Goethe` and `Peter`
are spelled correctly only because the compound checker takes them apart, so
their genitives are still missing.

#### Declining an already-declined form

`OQRST` are the five adjective declension endings and they belong on the base:
`klein` gives `kleine`, `kleinem`, `kleinen`, `kleiner`, `kleines`. Several
hundred entries were themselves one of those five and carried the flags anyway,
so they declined a second time and put `kleineree`, `vielee` and `höherere` into
the dictionary — every one a plausible typo of the real form.

Nine were worse. A comment had lost its `#`:

```
höherer/~~Jq - comprtve jectveOQRST
```

`c`, `e`, `j`, `m`, `o`, `p`, `r`, `t` and `v` are all real flags, so a
comparative adjective read as a noun, a verb in two tenses, and an adverb.

```bash
harper-core/src/language/german/scripts/fix_german_double_declension.py --forms forms.txt --apply
```

Two things make this safe to run. Most of what the doubled flags generate is not
junk — `abstoßenderem` is a real comparative and `abstoßend` produces it too,
through `U` — so the script expands the whole dictionary before and after and
keeps an entry as it is whenever the change would take a *hunspell-known* form
with it. And `OQRST` are in the properties table as well, so removing them
removes a part of speech: `O`, `Q`, `S` and `T` each carry an adjective reading
and `R` an adverb one. Those move to `J` and `r`, which generate nothing.

#### The headwords Harper could not reach

`dictionary.dict` is a subset of igerman98's word list, and the compound checker
papers over about half of the difference: a missing headword often decomposes
into parts Harper does have. The other half were simply reported as
misspellings — proper names (`Kalaschnikow`, `Caligula`, `Rijswijk`),
place-name derivations (`Ihringshausener`), and ordinary vocabulary (`Styropor`,
`Parataxe`, `Lokativ`, `Absonderlichkeit`).

`harper-core/src/language/german/scripts/add_german_missing_words.py` imports them. It asks **harper-cli itself**
which words are unreachable rather than reimplementing the decomposition, because
the decomposition is the thing being measured and a second copy would drift:

```bash
cargo build --release -p harper-cli --features harper-core/multilingual
harper-core/src/language/german/scripts/add_german_missing_words.py --apply
```

Entries are written with the bare noun property — no affix, no compound flag. A
freshly imported name has no vouched plural, and `mirror_hunspell_flag.py` is the
tool for adding one, with every generated form checked.

Two limits are deliberate:

- **Capitalized headwords only.** igerman98 also lists lower-case compounding
  stems that hunspell itself rejects as words (`entscheidungs`), and those must
  not become entries.
- **`--min-length 5`.** Any entry of three characters or more becomes a compound
  element, and a short one is also a plausible typo of a frequent word. Importing
  `Aa`, `Ahr`, `Alf` and `Abo` along with the rest cost several hundred
  detections in `just language-recall german`; dropping them keeps nine-tenths of
  the reference coverage for a tenth of that.

#### Abbreviations, and why the table is short

`ggf.`, `engl.`, `hg.`, `op.`, `var.` are ordinary German and were reported as
misspellings: the tokenizer hands the linter the letters without the full stop,
and `dictionary.dict` had no entry for them. igerman98 does not list them either,
so `harper-core/src/language/german/scripts/add_german_abbreviations.py` is a curated table carrying the
expansion of every entry it writes. Flag `2` is the abbreviation property, and
`GermanNounCapitalization` rejects anything holding it — `hg` must never be
"corrected" to `Hg`.

Adding one is not free, and two classes stay out:

- **Words that are also a German noun.** `Alb`, `Pol`, `Port`, `Ungar`, `Finn`.
  Giving the lower-case spelling the abbreviation flag silences a correct
  capitalization lint.
- **Words that are the start or the end of German words.** Any dictionary entry
  of three characters or more may act as a compound element, so `versch`
  (verschieden) made `verschwand` decompose into `versch` + `wand` — no longer a
  misspelling, a compound noun with a capitalization lint on it. Check both ends
  against the expanded word list before adding:

```python
a = "sen"
sum(1 for w in words if w.startswith(a) and w[len(a):] in words)   # 22
sum(1 for w in words if w.endswith(a) and w[:-len(a)] in words)    # 16641
```

`sen` (Senior) is the ending of every `-sen` plural and infinitive German has,
and on its own it let most of a hundred generated typos through
`just language-recall german`. `abb`, `geb`, `gest`, `anm`, `verh`, `aufl`,
`syn` and `kap` are out for the same reason, not because they are rare.

Excluding abbreviations from compounding outright was tried and is worse. The
German dictionary leans on loose compounding for proper-name coverage: `Leitha`,
`Omaha` and `Himmerland` are only words because `ha` is one, and the rule cost
fifty-odd new spelling errors on the corpus.

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
it, so `harper-core/src/language/german/scripts/mirror_hunspell_flag.py` copies that membership across and then
*checks* the result — a Harper flag is added only when the expanded form list
accepts every form the rule would generate for that entry.

```bash
unmunch /usr/share/hunspell/de_DE.dic /usr/share/hunspell/de_DE.aff > forms.txt
harper-core/src/language/german/scripts/mirror_hunspell_flag.py --forms forms.txt --from J --to 78 --apply
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
harper-core/src/language/german/scripts/mirror_hunspell_flag.py --forms forms.txt --to UW --prune --apply
harper-core/src/language/german/scripts/mirror_hunspell_flag.py --forms forms.txt --from C --to UW --apply
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
`6` carry determiner, pronoun and conjunction for exactly the same reason. `0`
went to the `-es` genitive and `s` to the strong preterite, and with those two
gone **no character is free in both tables** any more. The next rule has to
extend an existing flag, or start by freeing one.

```bash
python3 -c "import json, string; d=json.load(open('annotations.json')); \
  u=set(d['affixes'])|set(d['properties']); \
  print(''.join(c for c in string.digits+string.ascii_letters if c not in u) or 'none')"
```

Freeing one is easier than it sounds, because several flags are defined and
carried by nothing. Count what the dictionary actually uses before assuming a
character is spoken for:

```bash
python3 -c "import collections; c=collections.Counter(ch \
  for l in open('dictionary.dict', encoding='utf-8') if '/' in l.split('#')[0] \
  for ch in l.split('#')[0].strip().split('/',1)[1]); \
  print(sorted((n,ch) for ch,n in c.items())[:8])"
```

That is where the feminine derivation's `K` and `L` came from. Both were
uppercase compound interfixes (`-n` and `-en`) that **no entry carried** and that
the compound checker never read — it reads the lowercase `k` and `l`, as
`compound_checker.rs` spells out. `H` (compound, no interfix) went the same way, to the
proper-name genitive. `E` (the `ver-` prefix) and `n` (separable-prefix
participle) are still sitting there unused, so the next two rules have somewhere
to go.

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

`h` was in the same bind for a different reason, and was by a wide margin the
worst of them. It has no property twin, but it was both the `-st` verb affix and
the flag `CompoundChecker` reads as "may form compounds" — and as the second of
those it sits on nearly every vetted noun, so every one of them also generated
`arzneist`. It accounted for the largest single block of junk in the file.

It is now split the same way `N` was, with one extra twist: the affix moved to
`G` and `h` stayed behind as a **property with empty metadata**. It has to stay
declared — `CompoundChecker` reads the raw flag characters, so the entries keep
it — but it now generates nothing.

##### Splitting a colliding flag

`N` has been through this and is the worked example. Its affix and its property
were the same letter, so the `-es` suffix reached every entry tagged a noun —
the largest flag in the file — and generated `ergebnises` and `altertumes` for
the great majority of them. Dropping the flag was not an option: that is the noun
reading.

The fix is to move the *affix* to a free character and leave the property where
it is:

1. Copy the affix definition from `N` to `0` and delete `N` from `affixes`.
   `properties` is untouched, so every noun keeps its reading.
2. Re-establish membership from igerman98, which has the same rule on `T`:
   `mirror_hunspell_flag.py --from T --to 0`.

There is more room for this than it looks. A flag used as an *affix* only has to
be free in `properties`, and several affix letters are defined but unused — `E`,
`G` and the dead participle rules `k`, `l`, `m`, `n`. Repurposing one of those is
cheaper than claiming a digit.

Two of them are traps. `CompoundChecker::is_compound_flag` lower-cases before it
looks, so `H`, `K`, `L` and `O` are read as the compound markers `h`, `k`, `l`,
`o`: putting an affix on one of those silently changes what decomposes.

`M` went the simpler route — the affix was deleted outright. Its `-er` was billed
as a compound interfix, which `CompoundChecker` inserts at boundaries anyway, and
the handful of forms hunspell accepted turned out to be coincidences
(`Heiler` → `heilerer` is the comparative of *heil*, not a plural). `a`, the
genuine `-er` plural, was retargeted at hunspell's `SFX R` in the same pass.

##### When igerman98 has no headword to ask about

Harper stores plenty of entries igerman98 derives instead — `vergoldet` is an
entry here and a generated form there — so membership cannot always be borrowed.
`--all-entries` drops the membership gate and lets the form check decide alone.

Use it with `--together`, which treats `--to` as one paradigm and adds all of the
flags or none. Without that, each flag is judged alone and a noun picks up the
single adjective-declension flag its plural happens to match: `Arznei` gains `R`
on the strength of `arzneien`, and with it an adjective reading it should not
have.

##### The oracle has gaps, and pruning is where they bite

`--together` matters just as much when pruning, where it drops the set only if
*none* of it verifies. One failing form is weak evidence: igerman98 lists the
present tense for around two thousand verbs whose preterite it simply omits, and
`bräunte` is a German word whether or not `bräunen` carries hunspell's `Y`.

Adding and pruning are not symmetric for this reason. A missing form can only
ever cost you an addition you did not make — harmless. The same missing form,
read as grounds for pruning, deletes a word Harper had right.

The verb tense flags `d`, `f` and `i` were taken through this twice, and the
second answer reversed the first.

Judged on precision alone the prune looks bad: it costs sixty-odd false-positive
occurrences on the corpus — almost all proper names — and a tenth of a point of
reference coverage. That is what the first pass saw, and it left them alone.

With the recall harness the same change reads differently. Those flags sit on
entries that are already inflected forms, so `wurde` carried `f` and generated
`wurdee`; the prune takes `wurdee`, `einemm` and thousands like them out of the
dictionary. Injected-error recall rises by three points and typo detection by
two — several hundred more real mistakes caught, for sixty false alarms on names.

The lesson is the one in the section above: one metric will report an improvement
that is not there, and will also hide one that is.

## Five linters that were registered and did nothing

`german_adjective_agreement`, `german_case_usage`, `german_noun_declension`,
`german_pronoun_agreement` and `german_subject_verb_agreement` were in the lint
group and emitted **nothing** — not on the archived corpus, and not on the
examples in their own module documentation. Removing them from the group left the
corpus output byte-identical, and they have since been deleted; `git log` still
has them.

Two things kept that hidden.

The first was a shared bug: four of them walked `document.get_tokens()` and
compared `tokens[i]` with `tokens[i + 1]`, which is a word and the *space* after
it. They never saw two words at once. Fixing that only revealed the second
problem.

The second was that the logic underneath was never finished. Once
`german_subject_verb_agreement` could see two words it produced over twelve
thousand lints on edited prose, proposing `ist` → `istt`, `die` → `diet`,
`ein` → `eint`. The other four stayed silent even with the bug fixed.

`german_adjective_agreement` had six passing tests, which is the part worth
remembering: every one of them asserted that *correct* text produces no lints, or
checked the description string. None asserted that a wrong phrase is caught, so
they passed against a linter that did nothing at all.

> A test that only proves a rule is quiet proves nothing. Assert a firing.

Real agreement checking needs *case*. The dictionary still carries none, and
`german_preposition_case.rs` works around that rather than fixing it — see
[Case without a case-marked dictionary](#case-without-a-case-marked-dictionary).
Anything that has to know the *noun's* case still waits on the dictionary.

All five are annotated in place and left out of the group. Fixing one means
writing the agreement logic, not re-registering it.

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
| `german_fixed_nominalization.rs` | `im übrigen` → `im Übrigen`, `des öfteren`, `auf dem laufenden` | every word is a real word, and the noun-phrase chunker sees an attributive adjective |
| `german_subordinate_comma.rs` | the missing comma before `weil`, `obwohl`, `falls`, `bevor` | punctuation is not the spell checker's business |
| `german_year_preposition.rs` | `in 2024` → `2024` / `im Jahr 2024` | an anglicism made of two correct words |

| `german_preposition_case.rs` | `wegen dem Wetter` → `wegen des Wetters`, `für dem Kind`, `mit das Auto` | the words are all correct, and the case is a relation between two of them |

Comma placement is the most common mistake in written German and most of it needs
a parser, but not this part: these conjunctions open a subordinate clause and
nothing else. What the rule needs is a short list of exceptions, and the corpus
found all of them:

- a **capital letter** means it is a name — the corpus has *"im Kabinett Weil
  III"*, the Minister-President;
- an abbreviation's own full stop already separates the clauses — *"…, z. B.
  weil …"*, where the tokenizer keeps the dot on the token;
- a focus particle or coordinator carries the comma further left — *"Er kam, vor
  allem weil …"*, *"…, und weil …"*;
- a temporal modifier fuses with a *temporal* conjunction — *"noch bevor"*,
  *"kurz nachdem"*, *"je nachdem"*. That list is kept separate on purpose:
  *"Das Haus steht noch, obwohl es alt ist"* needs its comma, so `noch` cannot be
  a particle everywhere.

## Case without a case-marked dictionary

Of 109,557 German noun entries, 99.6% carry no gender and **none** carries a
case. That is why the five agreement linters above never worked, and it is not a
gap that can be closed cheaply. `german_preposition_case.rs` gets a large part of
the value anyway, by never asking the noun anything.

Both sides of the check come from `grammar/`, written out in Rust:

- `grammar/determiners.rs` — the determiner paradigms, about 200 forms over 18
  stems, generated from two ending tables.
- `grammar/prepositions.rs` — what each of ~100 prepositions governs.

The check is then LanguageTool's `retainAll`: the preposition names the cases it
allows, the determiner carries every case it can be read in, and an empty
intersection is the error. On a 572-article prose corpus it reports 15 times, of
which 3 are false positives; on a battery of 28 preposition-case errors it finds
24 against LanguageTool's 18, with neither reporting anything on 28 matched
correct sentences.

**Why the data is in Rust and not in `dictionary.dict`.** A dictionary entry
carries an `Agreement`, whose case, gender and number are three independent sets.
A determiner cannot be written that way: *der* is nominative masculine, dative
feminine, genitive feminine and genitive plural, but never *nominative feminine*,
and `case = {NOM, DAT, GEN}` beside `gender = {M, F}` claims exactly that
combination. What is needed is a list of fully specified readings. The flags
`p`, `u`, `v`, `w` in `annotations.json` are declared as case flags and set no
metadata; they are the shape that does not work.

**Reading the noun.** The noun narrows the determiner before the intersection is
taken: *den* is accusative masculine singular or dative plural, and *Freund* is
a singular, so *mit den Freund* has no dative left. Two restrictions were forced
by measurement, and both are about the data rather than the method:

* **Only the number is read, never the gender.** Too much of the gender is
  still wrong. Switching it on after the gender pass above still reported *in
  der Leber* five times, *bei der Angabe* three, *bei der Aussprache* three —
  27 new false positives, every one a noun whose recorded gender the corpus
  could not reach. The igerman98 route does not close it either: restricted to
  `-er` and `-el`, where it is sharpest, it proposes 77 corrections at about
  69% precision, which would inject two dozen fresh errors of the same kind.
* **Only a singular narrows anything.** German weak masculines — *Mensch*,
  *Philosoph*, *Patient*, *Laie*, *Gedanke* — spell the oblique singular exactly
  like the plural, and the dictionary records `Menschen` as a plural only.
  Trusting that reported *"für den Menschen"* 44 times in one corpus.

Finding the head is its own problem, and three guards came out of the corpus.
A candidate is rejected when it **touches a hyphen** (*aus den Natur- und
Geisteswissenschaften*), when it **ends in `-n` or `-s`** — every German dative
plural ends in `-n`, and most of them are missing from the dictionary, so
*Fischen* and *Berufen* resolve to entries recorded as singulars — and when the
**phrase does not end there**, which a following capitalized word (*mit den
Florida Keys*) or a following adjective (*bei den lange Zeit allein bekannten
Verfahren*) both show. The scan also stops at a second determiner, a preposition
or a conjunction, because *mit diesen in Konkurrenz* has no noun of its own.

With all of that, the corpus reports the same 15 times as before the noun was
read at all.

**The spelling says more than the entry.** German has one inflectional ending
left that is exceptionless: **the dative plural takes `-n`**. *den Freunden*,
*den Kindern*, *den Lehrern*. The only nouns exempt are those whose plural is
`-s` (*den Autos*), a Latin or Greek form (*den Korpora*, *den Termini*, *den
Mimiviridae*), or an acronym (*den NSAR*).

So a noun ending in none of those cannot be a dative plural, whatever the
dictionary knows. That is what makes *mit den Freund*, *mit den Lehrer*, *mit
seinen Bruder*, *mit den Zug* and *bei den Bäcker* reportable — all five of the
cases LanguageTool catches in this class — without any entry carrying a number,
a gender or a case. `bruder` and `zug` carry none of the three.

Five determiner forms are held back from it. *Zu diesen zählen Annegray,
Luxeuil und St. Gallen* — `diesen` is the whole phrase and the capitalized word
belongs to what follows. `diesen`, `jenen`, `welchen`, `solchen` and `manchen`
stand alone as freely as they introduce a noun; the article forms are pronouns
only in a relative clause, which a comma announces.

On the prose corpus the rule adds five reports, every one of them a real
mistake: *zu den Ameisenbäume*, *mit den Worte*, *zu den niederländischen
Notfallpläne*, *zu den bekanntesten Vertreter*, *zu den sultanistischen Regime*.
One false positive comes with them, and one older one goes.

**What it still cannot see.** *in den Haus*, *wir haben den Auto*, *Ein Frau
steht an der Tür* — all of these need gender on the noun, and half the frequent
nouns still have none. Verb rection (*Ich danke den Mann*) and direction after a
two-way preposition (*auf den Tisch*) are out of reach for LanguageTool too.

**Where the false positives came from.** All of them were homography, and the
corpus found each class. They are worth listing because every one is a trap for
the next rule that reads a closed word list:

| Class | Example | Fix |
|---|---|---|
| noun and acronym homographs | *für kurze **Zeit** den Thron*, *am **MIT** das*, *Liu **Bei** die Provinz* | a preposition is lower case unless it opens the sentence; `zeit`, `bar`, `je`, `ausschließlich`, `entsprechend`, `inklusive` are out of the table entirely |
| infinitive clauses | *um **dem** Leser … zu bieten* | the case belongs to the infinitive. `zu` is written *inside* a separable verb — *entgegenzuwirken* — so searching for it as a token catches barely half |
| postpositions | *seiner Ansicht **nach** eine*, *ihm **zufolge** das*, *von dort **aus** eine* | what follows starts a new phrase; recognized from the word in front |
| subordinating conjunctions | *während **das** Kind schlief* | the subject is nominative, so only a dative is reported after these |
| verb and pronoun homographs | *zu **sein***, *mit **ihr*** | bare *sein* and *ihr* are not in the determiner table |
| fixed coordinations | *von **ein** und demselben* | the determiner is uninflected before *und*, *oder*, *bis* |
| prenominal genitive | *in **des** Kaisers Namen* | correct after any preposition |

`german_fixed_nominalization.rs` is the one rule here that **cannot** be a Weir
rule, and for an instructive reason: Weir matches words case-insensitively, so a
rule for `im übrigen` would match the correct `Im Übrigen` too and rewrite it.
The linter matches the nominalized word exactly and only the words before it
case-insensitively, so a correct phrase is never touched.

It also has to look *right*. These phrases are nominalizations only when no noun
follows — *"im **Folgenden**"* against *"im folgenden **Jahr**"*, *"im
**Einzelnen**"* against *"im einzelnen **Fall**"* — and reading the ending rather
than the part of speech gets that wrong, because `werden` ends like a declined
adjective and would hide *"im Folgenden werden Beispiele genannt"*.

Its table holds only the phrases whose capital is **obligatory**. The 1996
reform capitalized the family but left the small letter standing as an equal
variant in a handful of them — `bei weitem`, `ohne weiteres`, `von neuem`,
`aufs neue`, `zum besten` — and the line between the two groups is not one a
reader can feel. The rule's doc comment carries the LanguageTool query that
settles a candidate; a phrase LanguageTool accepts in *both* spellings does not
belong in the table.

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
  `Addresse`, `nähmlich`, `wiederrum`, `Vorraus`, `der/die/das selbe`.
- **Doubled particles**: `als wie` after a comparative.

One family that needed no rule at all: the English genitive apostrophe
(`Peter's`, `Auto's`, `auf's`). The spell checker already rejects every one of
them and suggests the right form, and it leaves the genuinely correct `Hans'`
alone. Check before writing a rule.

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

### Full stops that do not end a sentence

`GermanSentenceCapitalization` reported the word after every abbreviation and
every ordinal — *"Ludwig II. **von** Savoyen"*, *"und ggf. **die** Verstärkung"*,
*"Bacteroides spec. **gehören**"*. On the archived corpus that was the **only**
thing it ever fired on, so every one of its lints was a false positive.

The period is not the rule's to judge: it belongs to the token before it. The
linter now looks left before flagging and stays quiet after a numeral, a Roman
numeral, or one of the abbreviations in `SENTENCE_INTERNAL_ABBREVIATIONS` —
which is worth extending whenever a new one shows up, because German prose cites
languages (`pol.`, `ahd.`), degrees (`Dr. theol.`) and taxonomy (`subsp.`)
constantly. It also requires a space after the full stop, so a host name split at
its own dot (`cassini.ehess`) is not read as a sentence break.

What is left is markdown list and infobox fragments, where the source really does
continue a line in lower case. That is a parsing artefact of the corpus, not a
German rule.

### Tokens that are not words, continued

A **capital inside** a short token marks it the same way: `gGmbH`, `UdSSR`,
`RoHS`, `VdS`, and the unit symbols `kV`, `dB`, `mA`, `CaO`. German orthography
has no word-internal capital, so declining to spell-check these costs nothing,
and each of them otherwise draws a suggestion list of pure noise. The length cap
here is six — past that, a stray capital is likelier a typo in a real compound
than an acronym.

## Subject–verb agreement, and why it needed no new data

The one agreement class German can decide without gender. The features were
already in the dictionary in the sense that matters: conjugation is **one affix
per ending**, and each ending names its person and number. The six flags simply
carried no morphology, so `annotations.json` was the whole data change.

    f  -e    1st singular, and 1st/3rd in Konjunktiv I
    G  -st   2nd singular
    i  -t    3rd singular and 2nd plural
    j  -en   1st and 3rd plural
    d  -te   1st and 3rd singular preterite
    e  -ten  1st and 3rd plural preterite

`s`, the strong preterite endings, stays empty: one flag builds `-st`, `-t`,
`-est` and `-et`, an affix rule has no metadata slot per replacement, and the
union of its persons says nothing.

The **subjects** and the **auxiliaries** are in `grammar/subjects.rs`, beside
the determiner and preposition tables and for the same reason. Personal
pronouns are a closed class of six forms. *sein*, *haben*, *werden*, the six
modals and *wissen* are irregular enough that the dictionary stores each form as
its own word, which means no affix and so no person — and they are the most
frequent verbs in the language.

### Four guards, each of which the corpus demanded

The first version reported **863 times** on 19 MB of edited prose. Every one was
a false positive and they came in four kinds. The rule reports none now.

* **Only the front field.** The pronoun has to open its clause — sentence start,
  or directly behind a comma or a coordinator. German is verb-second, so that is
  the one position where the finite verb is guaranteed to be the next word. In
  *das er vergessen hat* and *weil er gehen muss* the word behind the pronoun is
  a participle or an infinitive and the finite verb is at the end. This guard
  alone removed 846 of the 863.
* **No `es`.** German puts it in the front field as a placeholder while the real
  subject follows the verb: *Es werden fünf Klassen gebildet*, *Es existieren
  zahlreiche Ansätze*. Two hundred reports were this one word.
* **No adverb.** The `-st` affix is applied to adjective and pronoun roots too,
  so `selbst` and `möglichst` arrive carrying a second person singular. Sixteen
  of the last seventeen reports were that ending.
* **No capital.** A finite verb is never capitalized mid-sentence; what a
  capital marks behind a pronoun is an apposition — *wir Arbeiter*, *wir
  Deutsche*.

`ihr` is out of the pronoun table as well: it is a possessive and a dative far
more often than it is a subject.

### What it cannot see

*Du lernt* goes unreported. The `-t` ending is third singular and second plural,
the two axes are independent, and so the pair also admits second person
singular. Expressing that would need the list of fully specified readings the
determiner table uses; the ending is not worth one.

A noun-phrase subject — *Die Kinder spielt im Garten* — is not checked yet. The
number is available (the determiner table has it, and 28 000 noun entries gained
one), but the front-field test is not enough on its own: *Die Kinder sehe ich
jeden Tag* is an inversion where the noun phrase is the object.

## das / dass, and why only one direction of it

The most taught mistake in German writing, and the one where the two words are
structurally identical. `das` is an article, a demonstrative and a relative
pronoun; `dass` is a conjunction. Both are followed by a subject with the finite
verb at the end of the clause, so what follows can never settle it on its own.

`DasDass.weir` asks two questions at once:

* **What stands in front of the comma?** A verb of saying or thinking, a
  predicate adjective, or one of a short list of abstract nouns opens a content
  clause, which takes `dass`. A concrete noun opens a relative clause, which
  takes `das` — *das Buch, das er gelesen hat* stays as it is, because *Buch* is
  in none of the lists.
* **What follows?** Only a personal pronoun counts. A finite verb there (*ich
  glaube, das ist richtig*) or a capitalized noun (*ich glaube, das Buch ist
  gut*) means the demonstrative or the article, and both are left alone.

**Infinitives, participles and plural finite forms are not triggers**, and
leaving them in is what the first draft got wrong. German puts the verb last in
a subordinate clause, so *um das Erscheinungsbild zu verstehen, das sich …*,
*haben das Heil gesehen, das du …* and *auf ein Brötchen zeigt, das er kaufen
möchte* all put a verb of perceiving directly in front of a comma with a
relative clause behind it. That draft reported twelve times on the prose corpus
and was wrong every time. Only the singular finite forms, which stand in second
position, are safe. The price is that *ich habe gehört, dass …* goes unreported.

The reverse direction — `dass` written for the relative pronoun, *das Buch, dass
ich gelesen habe* — needs the antecedent's gender, because only a neuter noun
takes `das`. It is deliberately absent.

### Comma before `dass`

`dass` now sits in `GermanSubordinateComma`'s `SUBORDINATORS`, where it belongs:
it is the one conjunction in German with no second reading at all. Two things
had to come with it.

`DASS_MODIFIERS` holds the words that fuse with it into a two-part conjunction —
*ohne dass*, *statt dass*, *so dass*, *als dass*, *kaum dass*. The comma goes in
front of the pair; suggesting one after the first half produces *"ging ohne,
dass"*, which is wrong.

`is_bare_mention` covers the conjunction being named rather than used, which a
grammar article does constantly: *Subjunktionen sind vor allem dass und ob*,
*Inhaltssätze mit dass oder ob*. A conjunction in use is followed by the clause
it opens, so a coordinator or a bracket behind it means the word is one item in
a list of words. A second conjunction in front of it says the same — *wogegen
dass vor allem Aussagen markiert* has two in a row, which no German clause does.
The guard is not specific to `dass`: it also removed five standing false
positives on *weil* and *solange*. It costs six of the 515 injected missing
commas in `just language-recall german` — a comma directly behind the
conjunction reads as a list of words to it, and *weil, wie er sagte, …* is a
parenthesis rather than a list. Five wrong reports on edited prose against six
synthetic ones is close, and the narrower version that keeps only brackets and
coordinators was measured too: it saves the six and lets three of the five
back in.

### Measuring a new rule of this kind

```bash
./target/release/harper-cli test <path to the .weir file>     # its own tests
cargo build --release --bin harper-cli --features de
./target/release/harper-cli lint --dialect de --only DasDass \
    --format compact .archive/german-language/corpus-prose/*.md
```

The corpus is edited prose, so **every report it produces is a false positive**
and the target is zero. Read each one before changing anything: all twelve from
the first draft were the same construction, and the fix was to take three word
classes out of one list rather than to add a guard.

## Quoted English is the largest false-positive class

Measured against LanguageTool on the same prose: on a German Wikipedia article
Harper reports far more spelling mistakes than LanguageTool does, and the
difference is not a smaller dictionary. It is the bibliography. German academic
prose quotes constantly and mostly not in German, and a German dictionary has
nothing useful to say about *Molecular biology of the cell* or *Le marquage
différentiel de l'objet*.

`german_foreign_stretch` was written for the capitalization rules and asks for
evidence a dictionary lookup cannot give — function words the other language has
and German does not, counted in a window either side. `GermanSpellCheck` now
asks it too, before reporting anything.

Two things to know before touching it:

* **The window counts word tokens.** It used to count tokens, and a
  bibliography line is half punctuation — a colon, a comma, two brackets — so
  five tokens reached three words of English and the evidence fell short. The
  doc comment always said word tokens; the code did not.
* **The cost is stated in the tests.** A German misspelling standing *inside* an
  English run is no longer reported;
  `a_german_typo_inside_an_english_run_is_missed` says so on purpose. Measured
  against the injection harness the exchange is heavily one-sided, but it is an
  exchange.

### How to check whether a change here pays

```bash
cargo build --release --bin harper-cli --features de
./target/release/harper-cli lint --dialect de --only GermanSpellCheck \
    --format compact .archive/german-language/corpus-prose/*.md | wc -l
just language-recall german .archive/german-language/corpus-prose
```

Read a sample of the reports a change removes before believing the first number.
Roughly one report in eight on that corpus sits in a stretch with two or more
foreign function words nearby; the rest are proper names and technical
vocabulary that neither Harper nor LanguageTool knows, and no guard reaches
them.

### Comparing against LanguageTool

`docker start lt-bench`, then POST to `http://localhost:8010/v2/check` with
`language=de-DE`. Send the file **unpreprocessed** so its character offsets line
up with `harper-cli lint --format json`, and align the two by span overlap.
Stripping markdown headings first is what makes LanguageTool's `DE_CASE` fire
three hundred times: the heading runs into the next sentence and every sentence
opener looks like a capitalized word mid-sentence.

What the comparison turns up beyond spelling, in descending order of how often
it fires on clean prose:

* `EMPFOHLENE_ZUSAMMENSCHREIBUNG` — *in Frage* → *infrage*, *mit Hilfe* →
  *mithilfe*, *auf Grund* → *aufgrund*, *so genannt* → *sogenannt*. Harper has
  nothing for this class. Both spellings are allowed, so it belongs with the
  style rules, not the grammar ones.
* `DE_AGREEMENT` — *des Protein*, *zur Aufenthaltsbestimmungen*, *der
  Sachverständigenausschusses*. Real errors in edited text, and still blocked on
  the same gender data as everything else that reads the noun.
* `WHITESPACE_RULE`, `DOPPELTES_LEERZEICHEN`, `COMMA_PARENTHESIS_WHITESPACE` —
  typography, cheap, and absent.

### A measurement trap

German spelling suggestions are **not reproducible between runs**. `fuzzy_match`
returns at most a hundred candidates and which hundred depends on hash order, so
a candidate that would rank fourth is sometimes outside the set:

```bash
for i in 1 2 3; do ./target/release/harper-cli lint --dialect de \
    --only GermanSpellCheck "Reserpin" ; done
```

Diff two corpus runs on `file:line:column` and the flagged word, never on the
whole message, or the suggestion lists alone will invent hundreds of differences.

## The attribute that grows a phrase of its own

`GermanNounCapitalization` finds the head of a noun phrase by walking right from
the determiner over the attributive adjectives. A preposition ends that walk,
because a preposition really does end most noun phrases — *die Blume in dem
großen Garten* is finished at *Blume*.

German has one construction where it does not. The *erweitertes Attribut* hangs
a phrase in front of the adjective that modifies the head:

> die einzige **in Mitteleuropa** heimische Pflanzenart
> die ganze **nach links hinten** verlagerte Last
> eine weitere **von First Star Software** lizenzierte Spielausgabe

Stopping at the preposition crowns *einzige*, *ganze*, *weitere* and reports an
ordinary attributive adjective as a noun that lost its capital. Academic prose
is full of these, and they were the largest single class of capitalization false
positives with a describable cause.

Two conditions let the walk through, and both are needed, because an attribute
and a postmodifier look identical from the preposition onwards:

* the word in front of the preposition is an **adjective**. That is the whole
  difference: *die einzige in …* carries on to its head, *die Blume in …* does
  not.
* the attribute **closes the way an attribute must** — a lower-case adjective
  with a capitalized word directly behind it, within eight tokens and inside the
  sentence. The walk then resumes at that adjective rather than a token at a
  time, so the capitalized word *inside* the attribute (*Mitteleuropa*) does not
  become the head.

A lower-case head leaves the attribute unrecognised, because the second
condition cannot be met. That is the safe direction: *der hohe in der Stadt
stehende turm* still reports `turm`.

### Measuring a change to the chunker

Both directions have to be measured, and this change is a good illustration of
why: the reports it removes and the injected errors it stops catching came out
at the same count. They are not worth the same — the removals are on edited
prose a user would actually write, the losses are on synthetic injections — but
a change that only reports one of the two numbers is not measured.

```bash
cargo build --release --bin harper-cli --features de
# precision: every report on clean prose is a false positive
./target/release/harper-cli lint --dialect de --only GermanNounCapitalization \
    --format compact .archive/german-language/corpus-prose/*.md | wc -l
# recall: lower-case nouns injected into the same prose
just language-recall german .archive/german-language/corpus-prose
```

Read the reports the change removes before believing the first number. The
classes behind the remainder, counted on the same corpus: four in five sit
inside an ordinary German sentence, and the rest inside quoted English or a
bibliography, which `german_foreign_stretch` is for. Within the German ones the
recurring shapes are a finite verb whose entry is noun-only (*Dies würde*, *…
des Gehirns eintritt*), a preposition that is also a noun (*kraft*, *mangels*,
*samt*), and an indefinite pronoun German writes lower case (*die andere*, *eine
weitere*).

### Two levers that were measured and left alone

* **Widening `german_foreign_stretch`.** Only a small share of the remaining
  reports have even one foreign function word within five tokens, so the lever
  is close to exhausted; letting unknown words stand in for the second function
  word barely moves it, because the compound splitter makes English words known.
* **Giving declined adjectives their adjective reading in bulk.** `hunspell -m`
  names the rule it used, so every form igerman98 builds with `SFX A` can be
  found and the reading appended — around eleven thousand entries, and the
  sample reads perfectly. It removes very few reports, because the words that
  actually recur already have the reading, and it costs several times that many
  injected detections. The gap is real; closing it this way is not worth it.

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

### The word lists that are left, and why

A word list in a `const &[&str]` is almost always misplaced — it is vocabulary
pretending to be logic, and the dictionary is where vocabulary belongs. Audit
what is there before adding to it:

```bash
grep -rn 'const [A-Z_]*: &\[&str\]' linting/ spell/ | wc -l
```

`GERMAN_NON_NOUNS` was 265 words, justified on the grounds that "the dictionary
actively mistags them". That stopped being true when
`harper-core/src/language/german/scripts/strip_german_noun_readings.py` took the noun reading off every
lower-case entry igerman98 has no capitalized form for: 230 of those 265 words
stopped reading as nouns, and deleting them from the list changed no lint on
either corpus. It is 35 words now, and they are the part a dictionary cannot
settle — each really *is* a noun capitalized (`die Frage`, `die Waren`, `das
Gut`), so the entry is right and only the lower-case occurrence needs letting
through.

The rest stay, for three different reasons:

- **Closed grammatical classes**, which are the linter's reasoning and not
  vocabulary at all: `NOUN_PHRASE_LICENSORS` (the left-context test, consulted
  only after the `is_preposition`/`is_determiner` metadata fast path fails),
  `NP_BARE_PREPOSITIONS`, `RELATIVE_PRONOUNS`, `COORDINATORS`,
  `DEGREE_MODIFIERS`, `SUBORDINATORS`, `FOCUS_PARTICLES`.
- **Not German**: `FOREIGN_FUNCTION_WORDS` and `LANGUAGE_GLOSS_MARKERS` exist to
  recognise that a stretch of text is *not* German. A German dictionary is the
  wrong home for them by definition.
- **Not a membership test**: `SEPARABLE_VERB_PREFIXES` is matched as a *prefix*
  (`herausrückt`), which no dictionary lookup can do.

`SENTENCE_INTERNAL_ABBREVIATIONS` is the borderline one. It is 134 abbreviations
whose full stop does not end a sentence — arguably a property of each word, and
a flag could carry it. But 104 of them are three characters or more, so making
them entries would make them compound elements too, and the measured price of
short entries is typo detection every time (see the `-s` genitive above, and the
abbreviations that were kept out of `add_german_abbreviations.py` on the same
grounds). It is also a tokenizer question rather than a lexical one: the linter
asks it of the token before a period, not of a word in isolation.

**Note on flag characters**: many letters are simultaneously a property *and* an
affix rule, so adding a letter flag to a word can generate unintended forms
(tagging `mein` with the determiner flag `D` would also produce `meinung`). No
character is free in both tables, but several are *defined and carried by
nothing* — see "Affixes and properties share one namespace" for how to find
them.

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
are far and away the largest source of false positives this rule has. These
interruptions are stepped over rather than stopped on:

| In the text | Would otherwise stop at |
|---|---|
| *eine neue, radikalere Welle* | the comma, or `und`/`oder` |
| *der milde, **aber** wenig angenehme Pilz* | the contrastive conjunction |
| *der gerade oder **etwas** gekrümmte Griffel* | the degree word |
| *das beginnende **19.** Jahrhundert* | the numeral |
| *eine große, **1671** gefertigte Uhr*, *der lange **0,9 m** breite Gang* | the numeral, then the unit |
| *eine eigene **„**Baumnorm“*, *die deutsche **(**Wieder-)Besiedlung* | the quote or bracket |
| *eine reiche und **diversifizierte** Tierwelt* | an entry with no part of speech |

That last row is a class of its own. `diversifizierte` **is** in the dictionary,
carrying no reading at all — not a noun, not an adjective, nothing. It is not
`is_oov` either, so the walk treated it as a phrase boundary. An entry with no
part of speech is evidence of nothing and has to be passed over exactly like an
unknown word.

#### Where an adjective may be the head

Three grammar facts decide this, and getting any of them wrong turns an ordinary
adjective into a reported capitalization error:

1. **A nominalized adjective needs a determiner and carries a declension
   ending.** *das Gute*, *im Freien*, *für Deutsche*. So the **base** form under
   something that supplies no determiner — a bare preposition, a numeral — is
   predicative and never a noun: *weiß bis **braun***, *von **gelb** zu weiß*,
   *davon sind vier **unbewohnt***. This is why `NOUN_PHRASE_LICENSORS` and
   `NP_BARE_PREPOSITIONS` are two lists: `im`, `zum`, `zur` and the other fused
   forms *are* a determiner and stay on the licensor side. The declined form
   after a bare preposition is a real nominalization and keeps its head —
   dropping that distinction breaks *"nur für deutsche"*.

2. **German compounds are right-headed.** A *Determinativkompositum* takes its
   word class from the **last** element: *Stickstoff* + *tolerant* is an
   adjective. `CompoundChecker::get_compound_metadata` used to ask whether the
   *first* element was an adjective, which is the opposite question, and
   `stickstofftolerant` and `galleresistent` came back nouns. The first element
   still gets a say, but only as the tie-breaker when the head is itself a
   noun/adjective homograph: *purpur* + *braun* is a colour, *Bürger* + *recht*
   a noun.

3. **`der`/`die`/`das` after a comma is a relative pronoun.** German spells the
   article and the relative pronoun identically and punctuates every relative
   clause, so the comma is the signal. Read as an article it opens a phrase whose
   "head" is whatever the clause starts with — an adverb or a finite verb:
   *…, der **zuletzt** 2019…*, *…, die **angibt**, wie viele…*. The cost is a
   missed lint in *"das Haus, das große fenster hat"*, which is the trade this
   rule makes everywhere.

An ordinal is also where the **sentence segmenter** splits, so *das sowjetische
170. | Regiment* arrives here in two halves with the adjective last. A phrase
followed only by a numeral and a full stop is that split, and the adjective is
left uncrowned rather than made the head.

#### Quoted foreign titles

German prose names foreign works without translating them, and a bibliography is
mostly that. Several of those words are in the German dictionary with a noun
reading — `texts`, `model`, `period`, `zone`, `roman`, `charme` — so each is
reported as a lower-case German noun:

> The Modes of scepticism: ancient **texts** and modern **interpretations**
> Galien et la **philosophie**
> De la cave **au** grenier

`in_foreign_stretch` reads the neighbourhood, three word tokens either side.
`FOREIGN_FUNCTION_WORDS` lists function words of the languages German quotes —
English, French, Italian, Spanish, Latin — chosen so that **none of them is also
a German word**; `des`, `in`, `da`, `so` and `e` are left out for that reason
alone. One of those has to be adjacent, plus a second point from another function
word or an unknown word.

Two guards earn their place, and both were found by measuring rather than by
reading the rule:

- **Unknown words alone do not count.** German Wikipedia is full of proper names
  the dictionary lacks; letting two of them silence the rule cost a fifth of the
  injected lower-case nouns in `just language-recall german`.
- **A German determiner directly in front wins.** *"durch die Zeitschrift Le
  Mercure Galant"*, *"an der University of Virginia"* are German sentences that
  merely name something foreign. Without this the rule cost recall; with it, the
  corpus loses twenty-odd false positives at **no** measurable recall cost.

Latin and taxonomic runs carry no function words at all — *"Conspectus generum
avium"*, *"Mellisuga minima vielloti"* — and stay flagged. Widening the rule to
reach them is what the first guard above rules out.

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

### Precision is only half of it

The archived corpus is edited Wikipedia. It measures exactly one thing: whether
Harper stays quiet on correct prose. It is silent about whether a rule fires when
it should — and a rule that never fires at all scores perfectly by that measure.
That is how the dead `k`/`l`/`m`/`n` participle affixes survived as long as they
did.

```bash
just language-lint-sources german     # precision: does it stay quiet when right?
just language-recall german           # recall: does it speak up when wrong?
```

The second injects the mistakes German writers actually make — `garnicht`,
`seid Jahren`, `wiederspricht`, `größer wie`, `Standart`, a dropped epenthetic
`e` — into that same clean prose at known offsets, and reports what fraction
Harper flags. Read the two together: a rule that flags everything would score
100% on recall alone.

Add a class to `INJECTIONS` in `harper-core/src/language/german/scripts/german_recall_check.py` whenever you add
a rule. It is the cheapest way to find out that a rule has stopped working.

### How precise this rule actually is

Measure it before trusting it. Classify every `GermanNounCapitalization` lint on
the corpus by how hunspell knows the word: capitalized only means a noun and a
probable true positive, lower case only means it is not a noun and the lint is
wrong.

On edited prose the ratio is bad, and improving the dictionary has not moved it.
The recall side is no better: `just language-recall german` lower-cases the nouns
in that same prose, and the rule finds well under three quarters of them. Every
other rule in the directory scores full marks on that harness, so this is not a
measurement artefact — it is the one rule that is both noisy and incomplete.
The overwhelming majority of the lints are **declined adjectives standing in for
an elided noun** — *"die niedere und die hohe Gerichtsbarkeit"*, *"drei weitere,
die …"*, *"gegen neue oder Schneegreifer"*, *"um andere zu unterrichten"*.
German keeps those lower case, and they are structurally identical to the
nominalizations that must be capitalized (*"auf das Wesentliche"*, *"nur für
Deutsche"*). Part-of-speech tags and a shallow chunker cannot separate them;
that distinction is semantic.

Three attempts that did **not** pay off, so nobody repeats them:

- Requiring a strong nominalizer (`das`, `alles`, `nichts`, `etwas`) to the left.
  Only a fraction of the lints have one, and the rule's own tests demand that a
  preposition license nominalization too — which is where most of the false
  positives come from.
- Stripping the `N` property from the entries hunspell knows only in lower case.
  Correct in principle, and it changes nothing: the noun reading comes from the
  plural affixes' `base_metadata` and from the compound decomposition, not from
  `N`.
- Reading the compound's word class off its head rather than defaulting to noun.
  The words that need it are not compounds at all — `überschritt` is a prefixed
  verb that decomposes as `über` + `Schritt`, and a head-driven rule still calls
  that a noun. It needs typed elements, the same prerequisite as everything else
  about the splitter.
- Narrowing the blanket reject on `-en`/`-er`/`-es`/`-em` endings, which is what
  costs the rule most of its recall (`Hauptquartier`, `Aussehen`, `Kloster` are
  all waved through). Keeping only the unambiguously verbal endings buys eleven
  points of recall and multiplies the lints on *correct* prose nearly sixfold.
  The blanket reject is crude and it is earning its place.

That last one is only knowable because both halves are measurable now. Run
`just language-recall german` **and** `just language-lint-sources german` before
and after any change here; one of them alone will tell you a change is an
improvement when it is not.

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
derive_pos_fixes.py suspects.json         # -> harper-core/src/language/german/scripts/german_pos_fixes.tsv
harper-core/src/language/german/scripts/fix_german_pos_flags.py --apply   # append the missing readings
```

`harper-core/src/language/german/scripts/german_pos_fixes.tsv` **is** tracked — it is the record of which words
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
   `harper-core/src/language/german/scripts/fix_german_pos_flags.py`, whose additive pass *appends* the missing
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
  `harper-core/src/language/german/scripts/add_german_missing_verbs.py` closes the verb side of this; nouns have
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

  **A minimum length is not the fix**, and this has now been measured twice, the
  second time on all three axes.

  Raising it from three to four catches `Diskusion`, `Vorraussetzung` and the
  doubled-letter typos the splitter waves through (`einemm` is `eine` + `mm`,
  the millimetre). Recall and typo detection both improve slightly. It also
  markedly increases the lower-case false positives — the ones that actually
  interrupt a writer — because German builds just as freely on short *prefixes*:
  every one of the new ones was a `vor-` verb (`vorgesehen`, `vorgeschlagen`,
  `vorgenommen`).

  Allowing a curated list of prefixes back in recovers most of that, and then the
  list stops converging: the next round needs `-bar`, `rot`, `neo-`, `non-`, and
  so on without end. A threshold that needs a hand-maintained exception list to
  avoid regressions is a liability, not a fix. The elements have to be **typed** —
  prefix vs. noun — before any threshold helps.
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