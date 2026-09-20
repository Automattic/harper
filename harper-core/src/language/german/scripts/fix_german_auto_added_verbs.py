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
"""

import re
import sys

sys.path.insert(0, str(__import__("pathlib").Path(__file__).resolve().parent))

from german_dictionary_oracle import (  # noqa: E402
    DICT,
    analyse,
    argument_parser,
    check_umlauts_survive,
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
AUTO_ADDED = re.compile(r"\bauto-added\b")
INFINITIVE = re.compile(r"(en|ln|rn)$")


def capitalized(word):
    return word[:1].upper() + word[1:]


def verb_only(flags):
    """Flags that say `verb` and nothing else about the word class."""
    letters = flagset(flags)
    return VERB_PROPERTY in letters and letters <= (VERB_AFFIXES | {VERB_PROPERTY})


def candidates():
    for index, word, flags, comment in entries():
        if AUTO_ADDED.search(comment) and verb_only(flags):
            yield index, word, flags, comment


def is_noun_not_verb(word, stems):
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
    upper = capitalized(word)
    readings = stems.get(word, set()) | stems.get(upper, set())
    if not any(stem[:1].isupper() for stem in readings):
        return False
    return not any(INFINITIVE.search(stem) and stem.islower() for stem in readings)


def rewrite(flags):
    """Swap the verb flags for the noun ones, leaving anything else in place."""
    kept = [c for c in flags if c in "~*" or c not in (VERB_AFFIXES | {VERB_PROPERTY})]
    return "".join(kept) + NOUN_FLAGS


def main():
    parser = argument_parser(__doc__, "")
    args = parser.parse_args()
    check_umlauts_survive(args.hunspell_dict)

    found = list(candidates())
    print(f"{len(found)} auto-added entries carry a verb reading and nothing else")

    words = {word for _, word, _, _ in found}
    stems = analyse(words | {capitalized(w) for w in words}, args.hunspell_dict)

    changes = {
        index: (word, flags, rewrite(flags))
        for index, word, flags, _ in found
        if is_noun_not_verb(word, stems)
    }
    print(f"{len(changes)} of them are nouns hunspell knows, and no verb")
    for index in sorted(changes)[:15]:
        word, old, new = changes[index]
        print(f"    {word}/{old}  ->  {word}/{new}")

    if not args.apply:
        print("\nnothing written; pass --apply")
        return

    lines = DICT.read_text(encoding="utf-8").splitlines()
    for index, (word, old, new) in changes.items():
        body, sep, comment = lines[index].partition("#")
        lines[index] = f"{word}/{new}" + (f" {sep}{comment}" if sep else "")
    DICT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"\nwrote {DICT}")


main()
