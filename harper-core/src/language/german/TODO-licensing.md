# Open task: put the German dictionary on a licence-clean source

Status: open. Written at the end of the session that added the treebank genders
(commit `a6ffa29ad` and the docs commits after it), to hand the work to a new
session. Delete this file once the task is done.

## Background

**Licences in this repository.**

- **Harper as a whole:** Apache-2.0 (`LICENSE`, `harper-core/Cargo.toml`).
- **Slovak data:** MPL-2.0, inherited from hunspell-sk, which is MPL-2.0 only.
  It stays MPL-2.0. The per-file headers are all it needs, and they cannot be
  changed to Apache without the hunspell-sk authors' consent.
- **German data:** `dictionary.dict` and `annotations.json` carry no header,
  but they are largely derived from **igerman98 (GPLv2/GPLv3, Björn Jacke)**:
  - commit `da7f95314` imported 110K+ words from hunspell;
  - `scripts/add_german_missing_words.py` imports headwords and says "Harper's
    German dictionary is a subset of igerman98's";
  - `scripts/mirror_hunspell_flag.py` copies igerman98's flag membership;
  - several affix rules in `annotations.json` "mirror hunspell de_DE SFX …";
  - the header written by `scripts/extract_german_compound_begin_forms.py`
    names igerman98 as the source of `dictionary.dict`.

**Why it matters.** `dictionary.dict` is compiled into the binary
(`include_str!`), and `de` is a default feature of harper-cli, harper-ls and
harper-wasm. Every default build therefore carries GPL-derived data, and
anything distributed from it would have to be GPLv3 as a whole. That is
acceptable for a private fork, but not for Apache-licensed Harper upstream.

**What is not affected.** Facts about words (that a word exists, its part of
speech, gender and inflection) belong to no one. What GPL protects is
igerman98's specific selection and its flag assignments. All own work stays:
the affix rules as rewritten here, the hand corrections, the rules, tests and
batteries, and the treebank genders.

**Decision so far.** The owner does not want to negotiate a relicence. The
plan is an honest rebuild: re-derive the word list and flags from
permissively licensed sources, then document the new provenance truthfully,
including that earlier versions were derived from igerman98. Relabelling the
existing data as coming from another source without actually rebuilding it is
**not** an option: it would be false provenance, and the history names
igerman98 openly.

## Sources checked

| Source | Licence | Verdict |
|---|---|---|
| Wikidata Lexemes, as extracted in [kenix/lexicon-packs](https://github.com/kenix/lexicon-packs/releases/tag/lexicon-de-v3) `lexicon_de_grammar.db.gz` (7.4 MB gz, SQLite) | **CC0** | **Primary candidate.** 207,739 lemmas, 272,776 irregular forms, gender, part of speech. Regular forms deliberately dropped. SHA-256 gz `9393e9d52951308094b144bf7c39f6dcbfd69c96ac222420c714d5ca3423d055`, db `38d8079d3b1021c5a5b9a77256d10bcb438122e36e9eae88260a0a6c65994e27`. Do **not** use `lexicon_de_ipa.db.gz` from the same release, which is CC BY-SA. |
| Wikidata Lexemes directly (dumps.wikimedia.org, query.wikidata.org) | CC0 | Better and fuller, but blocked by the cloud environment's network policy in the last session. |
| Leipzig Corpora Collection, downloadable corpora | CC BY ([terms](https://wortschatz.uni-leipzig.de/en/usage)); other data CC BY-NC | Possible frequency/attestation source; host blocked in the cloud environment. Check that the specific file is a download corpus (CC BY). |
| UD German HDT/GSD treebanks | CC BY-SA 4.0 | Already used, read at runtime, flags only; the owner accepted this. Fine as a second attestation source. |
| FrequencyWords (hermitdave) | content CC BY-SA 4.0 (MIT is the code only) | No. |
| Wiktionary, MucLex, LanguageTool german-pos-dict | CC BY-SA | No. |

In the last session, downloading the kenix file worked, but reading it was
refused by the auto-mode permission check. Run the next session locally or
with a permission rule that allows reading the file. Keep downloaded data out
of git: put it under a directory listed in `.git/info/exclude`.

## Task

### Step 1: measure, change nothing yet

1. Load `lexicon_de_grammar.db` read-only and print its tables. It has a lemma
   table, a form→lemma table and `lemma_pos`.
2. For every entry of `harper-core/src/language/german/dictionary.dict`,
   classify it as follows:
   - **attested:** the word, or the lemma it is a form of, is a Wikidata lemma
     or form;
   - **attested via compound:** not itself in Wikidata, but every part the
     compound checker splits it into is;
   - **unattested:** neither.
3. Report the counts by type **and** weighted by frequency. Use the treebank
   texts or the prose corpus for the weights: a missing rare word costs far
   less than a missing frequent one.
4. Compare genders: Wikidata against `dictionary.dict` for the entries both
   have a gender for. Read every disagreement, as was done for the treebank
   genders (see "Ninth round" in `README.md`).
5. List the 200 most frequent unattested entries and read them. They decide
   how much of a setback the rebuild is.

Deliverable: a short report with these numbers and a recommendation.

### Step 2: rebuild (only after the owner decides on step 1)

- **Word list:** Wikidata lemmas, plus words attested in CC BY or own corpora,
  with the existing hand curation (`scripts/add_german_*`, `fix_german_*`)
  replayed on top.
- **Flags:** from Harper's own affix rules, with each generated form checked
  against forms attested in permissive sources, not against igerman98's form
  list. That means replacing `mirror_hunspell_flag.py`'s oracle.
- **Genders:** Wikidata (CC0), the article-cue oracle on own corpora, and the
  treebank genders.
- **Measure with the usual three axes** (precision corpus, injection, the
  batteries against LanguageTool) and compare against the last round's numbers
  in `README.md`.
- **Document the provenance truthfully,** in the README, the file headers and
  the commit message: "Since commit X the word list is built from Wikidata
  Lexemes (CC0) …; earlier versions were derived from igerman98 (GPL)."

## Not to do

- Do not relabel igerman98-derived data as coming from another source.
- Do not rewrite git history to hide the igerman98 origin.
- Do not change the Slovak licence headers.
- Do not commit Wikidata/treebank dumps or derived lists that carry a
  share-alike licence. CC0 material may be committed, with its source named.
