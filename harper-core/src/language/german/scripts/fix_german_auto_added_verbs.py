#!/usr/bin/env python3
"""Take the verb reading off the auto-added entries that are not verbs.

A bulk import once appended verb flags to thousands of entries on the strength
of their ending alone. Anything ending in `-en` looked like an infinitive, so
`hitlisten`, `schüben`, `gliedmaßen`, `eignungen` and `unwissen` are all filed
as verbs. None of them is one; every one is a noun form.

A wrong word class is not a cosmetic problem here, because two rules read it:

  * `GermanNounCapitalization` asks whether the word is a noun. `unwissen`
    carries only a verb reading, so writing "sein unwissen war groß" draws no
    lint at all — while `unheil`, `unsinn` and `unglück`, which kept their noun
    property, are reported correctly.

  * `can_head_a_lowercase_compound` accepts *any* word class, on the argument
    that a form the dictionary calls a verb was reached through a verb's own
    paradigm. A fabricated verb reading breaks that argument, and the word
    becomes a legal compound head. This is how `gen` — carrying `~~VijG # auto-added`
    though `Gen` is a noun — ends up heading 217 generated misspellings.

The oracle is `hunspell -m`, asked about the **capitalized** spelling, because
German capitalizes its nouns and so does hunspell's dictionary:

    Unwissen    st:Unwissen             a noun; `unwissen` has no reading at all
    Hitlisten   st:Hitliste fl:N        a noun plural
    Schüben     st:Schübe   fl:N        a noun plural
    Blonden     st:blond    fl:A        an adjective — the stem stays lower case
    Lesen       st:Lesen / st:lesen     both, and so left alone

An entry is rewritten only when hunspell reports a capitalized stem *and* no
infinitive, so a word that really is both keeps its verb. The verb property and
the verb affixes go; the noun property and the compound marker take their place,
which also drops the forms the verb affixes were inventing (`unwissene`,
`hitlistenen`).

The same import filed thousands of declined adjectives as verbs —
`abstrakten/~~VfjG`, `ostfränkischen/~~Vfj` — without the `auto-added` note.
Every reading hunspell has for them carries its adjective flag `A`, and none
names the word as its own lemma:

    ostfränkischen  st:ostfränkisch fl:A    an adjective form -> `~~J`

Those lose their verb flags for the adjective property, whatever their comment
says. The noun branch stays limited to `auto-added` entries.
"""

import re
import sys

sys.path.insert(0, str(__import__("pathlib").Path(__file__).resolve().parent))

from german_dictionary_oracle import (  # noqa: E402
    DICT,
    argument_parser,
    check_umlauts_survive,
    dictionary_encoding,
    entries,
    flagset,
)

VERB_PROPERTY = "V"
# The affix classes that only ever belong to a verb paradigm. `G` is the `-st`
# of the second person, which lives on a capital letter because the lower-case
# one was taken.
VERB_AFFIXES = set("cdefijklmnsG")
# What a noun form entry carries: the noun property and the compound marker.
NOUN_FLAGS = "Nh"
ADJECTIVE_FLAGS = "J"
AUTO_ADDED = re.compile(r"\bauto-added\b")
INFINITIVE = re.compile(r"(en|ln|rn)$")


def analyse(words, dictionary):
    """`{form: [fields, ...]}` from `hunspell -m`, without the compound parts.

    hunspell also analyses `brücken` as `st:brücken fl:k`: the form a noun takes
    *inside* a compound (`Brückenbau`). That stem is the word itself and shaped
    like an infinitive, so it would pass for one and keep the fake verb. Such an
    analysis says nothing about the word class and is skipped.
    """
    import collections
    import subprocess

    words = sorted(set(words))
    encoding = dictionary_encoding(dictionary)
    result = subprocess.run(
        ["hunspell", "-d", dictionary, "-m"],
        input=("\n".join(words) + "\n").encode(encoding, errors="replace"),
        capture_output=True,
        check=True,
    )
    out = collections.defaultdict(list)
    for line in result.stdout.decode(encoding, errors="replace").splitlines():
        form, _, analysis = line.partition(" ")
        fields = analysis.split()
        if not fields or "fl:k" in fields:
            continue
        out[form].append(fields)
    return out


def stems_of(analyses, form):
    """The lemmas hunspell names for `form`."""
    return {f[3:] for fields in analyses.get(form, ()) for f in fields if f.startswith("st:")}


def capitalized(word):
    return word[:1].upper() + word[1:]


def verb_only(flags):
    """Flags that say `verb` and nothing else about the word class."""
    letters = flagset(flags)
    return VERB_PROPERTY in letters and letters <= (VERB_AFFIXES | {VERB_PROPERTY})


def candidates():
    for index, word, flags, comment in entries():
        if word.islower() and verb_only(flags):
            yield index, word, flags, comment


def is_noun_not_verb(word, analyses):
    """Does hunspell call this a noun, and nothing that could be a verb?

    Two things have to hold. The capitalized spelling must analyse to a
    capitalized stem, which is the noun reading — German capitalizes its nouns
    and so does hunspell's dictionary. And **no** reading of either spelling may
    name a lemma shaped like an infinitive, because that is what a real verb
    entry looks like:

        Hitlisten  st:Hitliste   a noun, and `hitliste` is no infinitive
        Angaben    st:Angabe     likewise
        Anekeln    st:anekeln    `anekeln` *is* the infinitive: a real verb
        Eignungen  st:eignen     the lemma is a verb, so this is left alone

    The identity case has to count too. `anekeln` analyses to `st:anekeln`, the
    word itself; an earlier version excused that as "the stem is just the entry
    again" and turned the verb into a noun. Being one's own lemma is exactly
    what an infinitive does.
    """
    readings = stems_of(analyses, word) | stems_of(analyses, capitalized(word))
    if not any(stem[:1].isupper() for stem in readings):
        return False
    return not any(INFINITIVE.search(stem) and stem.islower() for stem in readings)


def is_adjective_form(word, analyses):
    """Is every hunspell reading of `word` an adjective form, and none its own lemma?"""
    readings = analyses.get(word)
    if not readings:
        return False
    return all("fl:A" in fields for fields in readings) and word not in stems_of(analyses, word)


def rewrite(flags, word_class):
    """Swap the verb flags for `word_class`, leaving anything else in place."""
    kept = [c for c in flags if c in "~*" or c not in (VERB_AFFIXES | {VERB_PROPERTY})]
    return "".join(kept) + word_class


def main():
    parser = argument_parser(__doc__, "")
    args = parser.parse_args()
    check_umlauts_survive(args.hunspell_dict)

    found = list(candidates())
    print(f"{len(found)} lower-case entries carry a verb reading and nothing else")

    words = {word for _, word, _, _ in found}
    analyses = analyse(words | {capitalized(w) for w in words}, args.hunspell_dict)

    nouns = {
        index: (word, flags, rewrite(flags, NOUN_FLAGS))
        for index, word, flags, comment in found
        if AUTO_ADDED.search(comment) and is_noun_not_verb(word, analyses)
    }
    adjectives = {
        index: (word, flags, rewrite(flags, ADJECTIVE_FLAGS))
        for index, word, flags, _ in found
        if index not in nouns and is_adjective_form(word, analyses)
    }
    print(f"{len(nouns)} auto-added ones are nouns hunspell knows, and no verb")
    print(f"{len(adjectives)} are adjective forms")
    changes = nouns | adjectives
    for index in sorted(changes)[:15]:
        word, old, new = changes[index]
        print(f"    {word}/{old}  ->  {word}/{new}")

    if not args.apply:
        print("\nnothing written; pass --apply")
        return

    lines = DICT.read_text(encoding="utf-8").splitlines()
    # `medien/~~Vj` and `medien/~~Vfj` both become `medien/~~Nh`; keep one.
    present = {line.partition("#")[0].strip() for line in lines}
    dropped = set()
    for index, (word, old, new) in sorted(changes.items()):
        body, sep, comment = lines[index].partition("#")
        if f"{word}/{new}" in present:
            dropped.add(index)
            continue
        present.add(f"{word}/{new}")
        lines[index] = f"{word}/{new}" + (f" {sep}{comment}" if sep else "")
    lines = [line for index, line in enumerate(lines) if index not in dropped]
    DICT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"\nwrote {DICT}")


main()
