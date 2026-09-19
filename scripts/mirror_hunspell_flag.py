#!/usr/bin/env python3
"""Hand a Harper affix flag to the entries igerman98 marks with its counterpart.

Harper's `annotations.json` mirrors several of hunspell de_DE's affix rules, but
mirroring the *rule* is only half the job: the flag has to reach the right
entries. igerman98 already knows which words take it, so this copies that
membership across, and then checks the result rather than trusting it — a Harper
flag is added only when the expanded hunspell form list accepts every form the
rule would generate for that entry.

    # -ung nominalization: hunspell's J, Harper's 7 (singular) and 8 (plural)
    scripts/mirror_hunspell_flag.py --forms forms.txt --from J --to 78

    # un- prefix: hunspell's U, Harper's 9
    scripts/mirror_hunspell_flag.py --forms forms.txt --from U --to 9

The forms are computed by reading the rule out of `annotations.json` and applying
it here, so this cannot drift from what Harper will actually generate.

Generate the form list once (it is large, and not committed):

    unmunch /usr/share/hunspell/de_DE.dic /usr/share/hunspell/de_DE.aff > forms.txt

Run from the repo root. `--apply` writes; the default is a dry run. Re-running is
idempotent.
"""

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from add_german_verb_conjugation_flags import (  # noqa: E402
    ANNOTATIONS,
    DICT,
    condition_matches,
    parse_condition,
)

DEFAULT_DIC = Path("/usr/share/hunspell/de_DE.dic")


def read_dic(dic: Path) -> str:
    """Decode `de_DE.dic`, whichever encoding it actually is.

    The shipped `de_DE.aff` declares `SET ISO8859-1`, but the `.dic` next to it
    is often a symlink to the frami variant, which is UTF-8. Trusting the
    declaration silently mangles every umlaut, and the script then reports no
    matches at all — because the entries never matched in the first place.
    """
    raw = dic.read_bytes()
    try:
        return raw.decode("utf-8")
    except UnicodeDecodeError:
        return raw.decode("iso-8859-1")


def lemmas_with(dic: Path, flag: str) -> set[str]:
    """Every `de_DE.dic` headword carrying `flag`."""
    out = set()
    for line in read_dic(dic).splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "/" not in line:
            continue
        word, _, flags = line.partition("/")
        if flag in flags:
            out.add(word.lower())
    return out


def load_rule(flag: str) -> tuple[str, list]:
    """(kind, [(condition, remove, add)]) for one Harper affix flag."""
    data = json.loads(ANNOTATIONS.read_text(encoding="utf-8"))
    expansion = data["affixes"][flag]
    return expansion["kind"], [
        (parse_condition(r["condition"]), r["remove"], r["add"])
        for r in expansion["replacements"]
    ]


def forms_for(word: str, kind: str, replacements: list) -> list[str]:
    """Every surface form one flag's replacements produce for `word`."""
    out = []
    for slots, strip, add in replacements:
        if kind == "prefix":
            # Conditions on a prefix rule describe the start of the word.
            if len(word) < len(slots) or not all(
                slot(c) for slot, c in zip(slots, word)
            ):
                continue
            if strip and not word.startswith(strip):
                continue
            out.append(add + word[len(strip) :])
        else:
            if not condition_matches(word, slots):
                continue
            if strip and not word.endswith(strip):
                continue
            out.append(word[: len(word) - len(strip)] if strip else word)
            out[-1] += add
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="write the changes")
    parser.add_argument("--forms", required=True, type=Path, help="`unmunch` output")
    parser.add_argument(
        "--from",
        dest="source",
        help="the de_DE.dic flag whose membership to copy (omit with --prune)",
    )
    parser.add_argument(
        "--to",
        dest="target",
        required=True,
        help="the Harper flag(s) to act on, as one string (e.g. '78')",
    )
    parser.add_argument(
        "--all-entries",
        action="store_true",
        help="consider every dictionary entry instead of only those igerman98 "
        "marks, letting the form check alone decide. For flags Harper has "
        "entries for that igerman98 has no headword for at all.",
    )
    parser.add_argument(
        "--together",
        action="store_true",
        help="treat --to as one paradigm. When adding, add all of the flags or "
        "none: judged alone, a noun picks up the single adjective-declension "
        "flag its plural happens to match. When pruning, drop the set only if "
        "none of it verifies, so one gap in igerman98 does not strip a tense.",
    )
    parser.add_argument(
        "--prune",
        action="store_true",
        help="remove the flag(s) where hunspell rejects a generated form, "
        "instead of adding them where it accepts every one",
    )
    parser.add_argument(
        "--only-capitalized",
        action="store_true",
        help=(
            "skip lower-case entries. A rule that reads as one thing on a proper "
            "name and another on a common noun wants this: hunspell's bare-'-s' "
            "SFX S is the genitive of a name (Goethes) but a plural on Autos, and "
            "handing it to all 45545 matching entries cost 184 detections in "
            "`just language-recall german` -- on `anderen -> annderen` and the "
            "like -- for 284 corpus false positives. Capitalized entries alone "
            "keep a third of the gain at a cost of one."
        ),
    )
    parser.add_argument("--dic", type=Path, default=DEFAULT_DIC)
    args = parser.parse_args()

    if not args.prune and not args.source and not args.all_entries:
        parser.error("give --from, --all-entries or --prune")

    for path in (DICT, ANNOTATIONS, args.forms, args.dic):
        if not path.exists():
            print(f"{path} not found -- run from the repo root", file=sys.stderr)
            return 1

    # Two views of the same list. The lower-cased one is what a lower-case entry
    # is checked against; the case-preserving one is what a capitalized entry is
    # checked against, and it is the only thing that keeps a verb flag off a
    # noun. `eingriff` is the preterite stem of `eingreifen` and carries SFX Z;
    # our entry for the homograph is the noun `Eingriff`. Verified in lower case
    # it passes on `eingriffst` and the noun starts generating `Eingriffst`.
    raw_forms = [
        line.strip()
        for line in args.forms.read_text(encoding="utf-8", errors="replace").splitlines()
        if line.strip()
    ]
    forms = {form.lower() for form in raw_forms}
    forms_cased = set(raw_forms)
    members = (
        set()
        if args.prune or args.all_entries
        else lemmas_with(args.dic, args.source)
    )
    rules = {flag: load_rule(flag) for flag in args.target}
    if args.prune:
        print(f"{len(forms)} hunspell forms, pruning '{args.target}'")
    elif args.all_entries:
        print(f"{len(forms)} hunspell forms, adding '{args.target}' wherever verified")
    else:
        print(
            f"{len(forms)} hunspell forms, {len(members)} lemmas carrying "
            f"'{args.source}', adding '{args.target}'"
        )

    out = []
    changed = 0
    unverified = 0
    samples = []

    for line in DICT.read_text(encoding="utf-8").splitlines(keepends=True):
        body, sep, comment = line.partition("#")
        stripped = body.strip()
        if not stripped or "/" not in stripped:
            out.append(line)
            continue

        word, _, flags = stripped.partition("/")
        lower = word.lower()

        def verified(flag: str) -> bool:
            kind, replacements = rules[flag]
            # Generate in the entry's own case, which is the case Harper will
            # generate in, and check that exact spelling. Lower-case entries fall
            # back to the lower-cased view: unmunch capitalizes sentence-initial
            # variants unevenly, so demanding an exact match there loses real
            # verbs for no reason.
            produced = forms_for(word, kind, replacements)
            if word.islower():
                return bool(produced) and all(f.lower() in forms for f in produced)
            return bool(produced) and all(f in forms_cased for f in produced)

        if args.prune:
            present = [f for f in args.target if f in flags]
            if args.together:
                # Drop the whole set only when none of it holds up. One failing
                # tense is more often a gap in igerman98 than evidence the entry
                # is not a verb: it lists the present for ~2000 verbs whose
                # preterite it omits, and "bräunte" is a word either way.
                doomed = present if present and not any(verified(f) for f in present) else []
            else:
                doomed = [f for f in present if not verified(f)]
            if not doomed:
                out.append(line)
                continue
            kept = "".join(c for c in flags if c not in doomed)
            trailing = body[len(body.rstrip()) :]
            out.append(f"{word}/{kept}{trailing}{sep}{comment}")
            changed += 1
            if len(samples) < 12:
                samples.append(f"{stripped}  ->  {word}/{kept}   (dropped {''.join(doomed)})")
            continue

        if not args.all_entries and lower not in members:
            out.append(line)
            continue

        if args.only_capitalized and not word[:1].isupper():
            out.append(line)
            continue

        if args.together:
            missing = (
                args.target
                if all(verified(f) for f in args.target)
                and any(f not in flags for f in args.target)
                else ""
            )
            missing = "".join(f for f in missing if f not in flags)
        else:
            missing = "".join(f for f in args.target if f not in flags and verified(f))

        if not missing:
            if any(flag not in flags for flag in args.target):
                unverified += 1
            out.append(line)
            continue

        new_body = f"{word}/{flags}{missing}"
        trailing = body[len(body.rstrip()) :]
        out.append(f"{new_body}{trailing}{sep}{comment}")
        changed += 1
        if len(samples) < 12:
            kind, replacements = rules[missing[0]]
            shown = ", ".join(forms_for(lower, kind, replacements)[:3])
            samples.append(f"{stripped}  ->  {new_body}   ({shown})")

    if args.prune:
        print(f"{changed} entries lose one or more of '{args.target}'")
    else:
        print(f"{changed} entries gain '{args.target}'")
        print(f"{unverified} skipped: hunspell does not accept every generated form")
    for s in samples:
        print(f"    {s}")

    if args.apply:
        DICT.write_text("".join(out), encoding="utf-8")
        print(f"wrote {DICT}")
    else:
        print("dry run; pass --apply to write")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
