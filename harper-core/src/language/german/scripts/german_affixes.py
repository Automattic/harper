#!/usr/bin/env python3
"""The affix machinery of `annotations.json`, in Python.

`expand(word, flags)` returns the forms Harper's loader generates for a
`dictionary.dict` entry, so the scripts that build or check that file can ask
"what would this flag set produce?" without compiling anything. It follows
`spell/rune/attribute_list.rs`: a suffix or prefix replacement fires when its
condition matches the end (start) of the word, `remove` must be there to be
cut, and a cross-product affix is applied again to every form it built with the
affixes of the opposite kind.

Only what the scripts need survives of the metadata: the parts of speech, and
the gender and number of a noun.
"""

import json
import pathlib

HERE = pathlib.Path(__file__).resolve().parent
ANNOTATIONS = HERE.parent / "annotations.json"

POS_KEYS = ("noun", "verb", "adjective", "adverb", "conjunction", "determiner", "pronoun")


class Info:
    """Parts of speech, gender and number a form carries."""

    __slots__ = ("pos", "gender", "number")

    def __init__(self):
        self.pos, self.gender, self.number = set(), set(), set()

    def merge(self, other):
        self.pos |= other.pos
        self.gender |= other.gender
        self.number |= other.number

    def __repr__(self):
        return f"Info({sorted(self.pos)},{sorted(self.gender)},{sorted(self.number)})"


def _as_set(v):
    if v is None:
        return set()
    return set(v) if isinstance(v, list) else {v}


def info_of(md):
    i = Info()
    for k in POS_KEYS:
        if md.get(k) is not None:
            i.pos.add(k)
    if md.get("preposition"):
        i.pos.add("preposition")
    noun = (md.get("morphology") or {}).get("noun") or {}
    i.gender |= {g.upper() for g in _as_set(noun.get("gender"))}
    i.number |= {n.upper() for n in _as_set(noun.get("number"))}
    return i


class Affixes:
    def __init__(self, path=ANNOTATIONS):
        a = json.load(open(path, encoding="utf-8"))
        self.affixes = {}
        for flag, v in a["affixes"].items():
            reps = [(r["remove"], r["add"], r["condition"]) for r in v["replacements"]]
            conds = [self._compile(c) for _, _, c in reps]
            target = Info()
            for t in v.get("target", []):
                if "if_base" not in t or t["if_base"] is None:
                    target.merge(info_of(t["metadata"]))
            self.affixes[flag] = {
                "prefix": v["kind"] == "prefix",
                "cross": v["cross_product"],
                "reps": [(r, ad, c, cl) for (r, ad, _), c, cl in zip(reps, [c for _, _, c in reps], conds)],
                "target": target,
                "base": info_of(v.get("base_metadata") or {}),
            }
        self.props = {}
        for flag, v in a["properties"].items():
            self.props[flag] = {
                "info": info_of(v["metadata"]),
                "propagate": v.get("propagate", False),
                "stem_only": v.get("stem_only", False),
            }

    @staticmethod
    def _compile(cond):
        """A condition as a list of (negated, charset | None) per position."""
        ops, i = [], 0
        while i < len(cond):
            c = cond[i]
            if c == "[":
                j = cond.index("]", i)
                body = cond[i + 1 : j]
                neg = body.startswith("^")
                ops.append((neg, set(body[1:] if neg else body)))
                i = j + 1
            elif c == ".":
                ops.append((True, set()))
                i += 1
            else:
                ops.append((False, {c}))
                i += 1
        return ops

    @staticmethod
    def _match(ops, seg):
        if len(ops) != len(seg):
            return False
        for (neg, cs), ch in zip(ops, seg):
            if neg == (ch in cs):
                return False
        return True

    def apply(self, flag, word):
        """The forms `flag` builds from `word`, with the target info."""
        aff = self.affixes[flag]
        out = {}
        for remove, add, cond_src, cond in aff["reps"]:
            n = len(cond)
            if n > len(word):
                continue
            if aff["prefix"]:
                if not self._match(cond, word[:n]):
                    continue
                if remove and not word.startswith(remove):
                    continue
                new = add + word[len(remove):]
            else:
                if not self._match(cond, word[len(word) - n :]):
                    continue
                if remove and not word.endswith(remove):
                    continue
                new = word[: len(word) - len(remove)] + add
            out[new] = aff["target"]
        return out

    def expand(self, word, flags, _into=None):
        """form -> Info for every form the entry yields, itself included."""
        res = {} if _into is None else _into
        base = Info()
        stem_only = False
        for f in flags:
            p = self.props.get(f)
            if p:
                base.merge(p["info"])
                stem_only |= p["stem_only"]
        for f in flags:
            aff = self.affixes.get(f)
            if not aff:
                continue
            base.merge(aff["base"])
            built = self.apply(f, word)
            if aff["cross"]:
                opposite = [
                    g for g in flags
                    if (g in self.props and (aff["prefix"] or self.props[g]["propagate"]) and not self.props[g]["stem_only"])
                    or (g in self.affixes and self.affixes[g]["prefix"] != aff["prefix"])
                ]
                for new, info in built.items():
                    self.expand(new, opposite, res)
                    res.setdefault(new, Info()).merge(info)
            else:
                for new, info in built.items():
                    res.setdefault(new, Info()).merge(info)
        if not stem_only:
            res.setdefault(word, Info()).merge(base)
        return res


def parse_dict_line(line):
    """(word, flags) of a dictionary.dict line, or None for a comment."""
    line = line.rstrip("\n")
    if not line or line.startswith("#"):
        return None
    body = line.split(" #", 1)[0].strip()
    if "/" in body:
        w, fl = body.split("/", 1)
    else:
        w, fl = body, ""
    return w, fl.replace("~", "")


if __name__ == "__main__":
    import sys

    aff = Affixes()
    for entry in sys.argv[1:]:
        w, _, fl = entry.partition("/")
        for form, info in sorted(aff.expand(w, fl).items()):
            print(form, info)
