#!/usr/bin/env python3
"""Mark the German entries that are bare stems rather than words.

A German verb is stored twice: once as the infinitive, and once as the stem the
conjugation affixes attach to. `absperr/~~Vcej` builds *absperrt*, *absperrte*
and *absperren*, and its line says so in a `# REPLACES absperren` comment. But
the entry itself also lands in the word list, and `absperr` is not a German
word.

That costs twice over. The stem is accepted as a spelling on its own, and —
because **every** member of the word list of three characters or more may be a
compound element — it turns up inside misspellings:

    Absiecht  = ab + siecht          (siecht: stem of *siechen*)
    Angebbot  = angeb + bot          (angeb: stem of *angeben*)
    altehr    = alt + ehr            (ehr:   stem of *ehren*)
    anallog   = anal + log

Marking the entry `*` keeps every conjugated form and drops the bare stem.

**Not every stem is a non-word.** German's imperative singular *is* the bare
stem: *geh!*, *hab!*, *werd!*. So the decision is not made from the `REPLACES`
comment but from an oracle:

    mark the entry only if every reading `hunspell -m` gives it
    is a bound one

`hunspell -l` is not enough — it accepts `absperr`, because its own dictionary
holds the same fragment for the same reason Harper does. The morphological
analysis distinguishes them, and it does so because the German hunspell
dictionary marks these entries `NEEDAFFIX` by hand:

    sperr      st:sperr fl:k        st:sperren fl:W    -> imperative, a word
    quer       st:quer              st:quer fl:k       -> a plain entry, a word
    absperr    st:absperr fl:k                         -> bound, not a word
    erd        (no reading at all)                     -> not a word

A reading with no flags is a plain dictionary entry, and a reading whose stem
is some other word is an inflected form; either makes the entry a word. An
entry that only ever comes back as its own bound stem does not.

The capitalized spelling is asked about too, because German writes its nouns
with a capital and so does hunspell's dictionary: `harz` is only a verb stem
there, while `Harz` is the mountain range. Asking about the lower-case form
alone marks every noun that shares its spelling with a stem.

Every line for a marked spelling is marked, not only the one with the
`REPLACES` comment. `absperr` is a stem twice over: the word list also carries
`absperr/~~NXh #noun`, from a pass that mined parts of speech out of a corpus,
and one unmarked line is enough to put the spelling back.

    harper-core/src/language/german/scripts/mark_german_stems.py              # report only
    harper-core/src/language/german/scripts/mark_german_stems.py --apply      # write dictionary.dict

Needs a **UTF-8** German hunspell dictionary; the one in `/usr/share` is
ISO-8859-1 and loses every umlaut, so the script checks and refuses.
"""

import argparse
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[5]
DICT = ROOT / "harper-core/src/language/german/dictionary.dict"

STEM_FLAG = "*"
# The comment the dictionary already uses to say "this line is a stem".
REPLACES = re.compile(r"#\s*REPLACES\s+(\S+)")


def capitalized(word):
    return word[:1].upper() + word[1:]


def all_entries():
    """`(line index, word, flags, comment)` for every flagged dictionary line."""
    for index, line in enumerate(DICT.read_text(encoding="utf-8").splitlines()):
        body, sep, comment = line.partition("#")
        word, _, flags = body.strip().partition("/")
        if word and flags:
            yield index, word, flags, comment.strip()


def entries():
    """The lines that declare a stem, by their `REPLACES` comment."""
    for index, line in enumerate(DICT.read_text(encoding="utf-8").splitlines()):
        if not REPLACES.search(line):
            continue
        body, _, comment = line.partition("#")
        word, _, flags = body.strip().partition("/")
        if word and flags:
            yield index, word, flags, comment.strip()


def dictionary_encoding(name):
    """The character set the hunspell dictionary declares, from its `.aff`."""
    candidates = [pathlib.Path(f"{name}.aff")] + [
        pathlib.Path(d) / f"{name}.aff" for d in ("/usr/share/hunspell", "/usr/share/myspell")
    ]
    for path in candidates:
        if path.exists():
            for line in path.read_text(encoding="latin-1").splitlines():
                if line.startswith("SET "):
                    return line.split(None, 1)[1].strip().replace("ISO8859", "iso-8859")
    return "utf-8"


def free_words(words, dictionary):
    """The subset of `words` hunspell gives at least one free reading.

    `hunspell -m` prints one line per reading: `st:` names the lemma and `fl:`
    the flags of the entry it came from. A reading with no `fl:` is a plain
    entry (`quer -> st:quer`), and a reading naming another lemma is an
    inflected form (`sperr -> st:sperren fl:W`) — both make the word free. A
    fragment that only ever comes back as its own flagged stem (`absperr ->
    st:absperr fl:k`), or that gets no reading at all (`erd`), is bound.
    """
    words = sorted(set(words))
    if not words:
        return set()
    encoding = dictionary_encoding(dictionary)
    result = subprocess.run(
        ["hunspell", "-d", dictionary, "-m"],
        input=("\n".join(words) + "\n").encode(encoding, errors="replace"),
        capture_output=True,
        check=True,
    )
    out = set()
    for line in result.stdout.decode(encoding, errors="replace").splitlines():
        form, _, analysis = line.partition(" ")
        fields = analysis.split()
        stems = [f[3:] for f in fields if f.startswith("st:")]
        if not stems:
            continue
        flagged = any(f.startswith("fl:") for f in fields)
        # Case alone is not a different lemma: `Absperr` analyses as
        # `st:absperr` only because a sentence may start with it.
        other = any(stem.lower() != form.lower() for stem in stems)
        if not flagged or other:
            out.add(form.lower())
    return out


def check_umlauts_survive(dictionary):
    """Refuse to run against a dictionary that mangles umlauts.

    `/usr/share/hunspell/de_DE.aff` declares ISO-8859-1, and on a UTF-8 system
    hunspell truncates every word at its first umlaut. Nothing errors; the
    oracle simply calls every umlaut word unknown, and every umlaut stem would
    be marked whether or not it is a word.
    """
    if free_words(["überschreitet"], dictionary):
        return
    raise SystemExit(
        f"hunspell -d {dictionary} loses umlauts: 'überschreitet' is not analysed.\n"
        "Pass --hunspell-dict with a path to a UTF-8 copy of the dictionary, "
        "e.g. LanguageTool's:\n  .../resource/de/hunspell/de_DE"
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="write dictionary.dict")
    parser.add_argument(
        "--hunspell-dict",
        default="de_DE",
        help="hunspell dictionary to consult; pass a path to a UTF-8 copy if "
        "the system one is ISO-8859-1",
    )
    args = parser.parse_args()

    check_umlauts_survive(args.hunspell_dict)

    declared = {word for _, word, _, _ in entries()}
    free = free_words(set(declared) | {capitalized(w) for w in declared}, args.hunspell_dict)
    bound = {word for word in declared if word.lower() not in free}

    # Every line for a bound spelling, not only the one carrying the comment:
    # `absperr` is declared a stem once and tagged `#noun` on a second line,
    # and one unmarked line is enough to put the spelling back in the list.
    marked = [e for e in all_entries() if e[1] in bound and STEM_FLAG not in e[2]]
    kept = sorted(declared - bound)

    print(f"{len(declared)} spellings are declared a stem")
    print(f"  {len(bound)} have no free reading -> mark every line for them")
    print(f"  {len(kept)} are a word in their own right -> leave")
    print(f"\n{len(marked)} lines to change")
    print("marked, sample:   " + ", ".join(w for _, w, _, _ in marked[:8]))
    print("left alone:       " + ", ".join(kept[:8]))

    if not args.apply:
        print("\ndry run; pass --apply to write")
        return 0

    lines = DICT.read_text(encoding="utf-8").splitlines()
    for index, word, flags, _ in marked:
        body, sep, comment = lines[index].partition("#")
        _, _, existing = body.rstrip().partition("/")
        lines[index] = f"{word}/{STEM_FLAG}{existing}" + (f" {sep}{comment}" if sep else "")
    DICT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"\nwrote {DICT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
