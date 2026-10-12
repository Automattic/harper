#!/usr/bin/env python3
"""German closed word classes, written out from grammar rather than taken from a word list.

Determiners, pronouns, numerals, the umlauting adjectives and the pronominal
adverbs are small, fixed classes that every German grammar lists. They are the
one part of `dictionary.dict` that is spelled out here by hand;
`build_german_dictionary.py` calls `entries()` and adds whatever Wikidata
knows on top.

`entries()` yields `(word, flags)`. Flags are those of `annotations.json`:
`4` determiner, `5` pronoun, `6` conjunction, `P` preposition, `r` adverb,
`1` spelled-out numeral, `2` unit abbreviation, `3` foreign term, `J` adjective,
`O Q R S T` the adjective endings.
"""

ADJ_ENDINGS = "OQRST"


def _forms(stem, endings):
    return [stem + e for e in endings]


def determiners():
    out = []
    # the definite article and the demonstrative pronoun that shares its forms
    for w in "der die das den dem des".split():
        out.append((w, "4"))
    for w in "dessen deren denen derer".split():
        out.append((w, "5"))
    # ein-words: article, possessive, negation
    for stem in ("ein", "kein", "mein", "dein", "sein", "ihr", "Ihr", "unser", "euer"):
        if stem == "euer":
            forms = ["euer"] + _forms("eur", ["e", "em", "en", "er", "es"])
        elif stem == "unser":
            forms = ["unser"] + _forms("unser", ["e", "em", "en", "er", "es"]) + _forms("unsr", ["e", "em", "en", "er", "es"])
        else:
            forms = [stem] + _forms(stem, ["e", "em", "en", "er", "es"])
        for f in forms:
            out.append((f, "45"))
        out.append((stem + "s", "5"))
    # strong-ending determiners: dieser, jener, jeder, welcher, mancher, solcher, aller ...
    for stem in ("dies", "jen", "jed", "welch", "manch", "solch", "all", "sämtlich", "etlich", "einig", "mehrer",
                 "beid", "irgendwelch", "irgendein", "jegliche"):
        if stem == "irgendein":
            forms = [stem] + _forms(stem, ["e", "em", "en", "er", "es"])
        elif stem == "jegliche":
            forms = ["jeglich" + e for e in ["e", "em", "en", "er", "es"]]
        else:
            forms = _forms(stem, ["e", "em", "en", "er", "es"])
        for f in forms:
            out.append((f, "45"))
    for w in "dies welch manch solch all jed jeglich irgendwelch".split():
        out.append((w, "4"))
    # derselbe, derjenige
    for first in ("der", "die", "das", "den", "dem", "des"):
        pass
    for art, endings in (
        ("der", ["", ]), 
    ):
        pass
    selbe = {
        "derselbe": "derselbe", "dieselbe": "dieselbe", "dasselbe": "dasselbe", "denselben": "denselben",
        "demselben": "demselben", "desselben": "desselben", "derselben": "derselben", "dieselben": "dieselben",
    }
    for w in selbe:
        out.append((w, "45"))
    for w in ("derjenige", "diejenige", "dasjenige", "denjenigen", "demjenigen", "desjenigen", "derjenigen",
              "diejenigen"):
        out.append((w, "45"))
    # quantifiers that decline like adjectives
    for stem in ("viel", "wenig", "ander", "mehr", "meist", "genug", "ganz", "halb", "selb", "selber", "sonstig",
                 "übrig", "gewiss", "bestimmt", "verschieden", "zahlreich", "einzeln", "gesamt", "jeglich"):
        out.append((stem, "4J" + ADJ_ENDINGS if stem not in ("mehr", "genug") else "4"))
    out += [("mehr", "4r"), ("genug", "4r"), ("viele", "45"), ("vielen", "45"), ("vieler", "45"), ("vielem", "45"),
            ("wenige", "45"), ("wenigen", "45"), ("weniger", "45r"), ("andere", "45"), ("anderen", "45"),
            ("anderer", "45"), ("anderem", "45"), ("anderes", "45")]
    return out


def pronouns():
    out = []
    for w in ("ich mich mir meiner du dich dir deiner er ihn ihm seiner sie ihr ihrer es ihnen euch uns unser wir "
              "Sie Ihnen Ihrer sich man jemand jemanden jemandem jemandes niemand niemanden niemandem niemandes "
              "wer wen wem wessen was nichts etwas einander jedermann jedermanns jeder irgendwer irgendwas "
              "irgendjemand irgendwen irgendwem welche welcher welches welchen welchem "
              "derselbe selbst mancher manche manches einer eine eines einem einen keiner keine keines keinem "
              "keinen meins deins seins ihrs unsers eures "
              "dieser diese dieses diesen diesem jener jene jenes jenen jenem").split():
        out.append((w, "5"))
    return out


def conjunctions():
    words = ("und oder aber denn sondern doch sowie sowohl weder noch entweder dass daß ob wenn weil da obwohl "
             "obgleich obschon während bevor nachdem sobald solange sofern falls damit indem sodass so dass "
             "ehe bis seit seitdem als wie wohingegen wogegen zumal bzw beziehungsweise sprich respektive "
             "ohne statt anstatt außer").split()
    return [(w, "6") for w in words if " " not in w]


def prepositions():
    words = ("an auf aus bei bis durch für gegen in mit nach ohne um unter von vor zu über hinter neben zwischen "
             "ab außer binnen dank entgegen entlang gemäß gegenüber innerhalb außerhalb oberhalb unterhalb "
             "diesseits jenseits laut mittels per pro seit statt anstatt trotz während wegen zufolge zuliebe "
             "samt nebst nahe fern längs links rechts unweit anhand anlässlich aufgrund bezüglich hinsichtlich "
             "infolge zwecks seitens ungeachtet vermittels vorbehaltlich zugunsten zuungunsten zuzüglich abzüglich "
             "einschließlich ausschließlich inklusive exklusive sonder pro via zwischen jenseits diesseits "
             "an auf ").split()
    out = [(w, "P") for w in words]
    contractions = ("am im zum zur beim vom ins ans aufs fürs ums durchs übers unters hinterm hinters vorm vors "
                    "überm unterm übers ums beim aufm ").split()
    out += [(w, "P4") for w in contractions]
    return out


def numerals():
    ones = ["null", "ein", "zwei", "drei", "vier", "fünf", "sechs", "sieben", "acht", "neun", "zehn", "elf", "zwölf",
            "dreizehn", "vierzehn", "fünfzehn", "sechzehn", "siebzehn", "achtzehn", "neunzehn"]
    tens = {2: "zwanzig", 3: "dreißig", 4: "vierzig", 5: "fünfzig", 6: "sechzig", 7: "siebzig", 8: "achtzig",
            9: "neunzig"}
    words = set(ones) | {"eins", "eine", "einen", "einem", "einer", "eines", "hundert", "tausend", "million",
                         "millionen", "milliarde", "milliarden", "billion", "billionen", "dutzend", "paar", "zig"}
    words |= set(tens.values())
    for t, tw in tens.items():
        for u in range(1, 10):
            unit = "ein" if u == 1 else ones[u]
            words.add(f"{unit}und{tw}")
    for h in range(1, 10):
        pre = "" if h == 1 else ones[h]
        words.add(f"{pre}hundert")
        words.add(f"{pre}tausend")
        for rest in list(range(1, 20)) + [20, 30, 40, 50, 60, 70, 80, 90]:
            r = ones[rest] if rest < 20 else tens[rest // 10]
            words.add(f"{pre}hundert{r}")
    out = [(w, "1") for w in sorted(words)]
    # ordinals: declined like adjectives
    ordinal = {"erst": 1, "zweit": 2, "dritt": 3, "viert": 4, "fünft": 5, "sechst": 6, "siebt": 7, "acht": 8,
               "neunt": 9, "zehnt": 10, "elft": 11, "zwölft": 12}
    stems = list(ordinal)
    for n in range(13, 20):
        stems.append(ones[n] + "t")
    for t, tw in tens.items():
        stems.append(tw + "st")
        for u in range(1, 10):
            unit = "ein" if u == 1 else ones[u]
            stems.append(f"{unit}und{tw}st")
    stems += ["hundertst", "tausendst", "millionst", "letzt", "nächst", "vorletzt", "zigst"]
    for s in stems:
        out.append((s, "J" + ADJ_ENDINGS))
    return out


# adjectives whose comparative and superlative take an umlaut
UMLAUT_ADJECTIVES = ("alt arm dumm gesund grob groß hart jung kalt klug krank kurz lang nah scharf schwach schwarz "
                     "stark warm blass fromm glatt gesund zart rot nass naß schmal").split()


def umlaut(word):
    """`alt` -> `ält`: the last a, o or u of the stem takes the umlaut."""
    for i in range(len(word) - 1, -1, -1):
        c = word[i]
        if c in "aou":
            if c == "a" and i + 1 < len(word) and word[i + 1] == "u":
                return word[:i] + "äu" + word[i + 2 :]
            return word[:i] + {"a": "ä", "o": "ö", "u": "ü"}[c] + word[i + 1 :]
    return word


def umlaut_degrees(adj):
    stem = umlaut(adj)
    comp = stem + "er"
    sup = stem + ("est" if adj.endswith(("t", "d", "s", "ß", "z", "x")) else "st")
    return comp, sup


# Pronominal and other adverbs a word list tends to miss.
ADVERBS = ("dabei dafür dagegen daher dahin dahinter damals danach daneben dann daran darauf daraus darin darüber "
           "darum darunter davon davor dazu dazwischen hierbei hierfür hiergegen hierher hierin hiermit hiernach "
           "hierüber hierum hierunter hiervon hiervor hierzu wobei wofür wogegen woher wohin womit wonach woran "
           "worauf woraus worin worüber worum worunter wovon wovor wozu nicht nur auch noch schon sehr immer nie "
           "niemals oft selten hier da dort jetzt nun heute morgen gestern bald dann doch ja nein vielleicht wohl "
           "eben halt eigentlich ziemlich fast gar etwa sogar besonders insbesondere bereits bisher bislang erst "
           "wieder weiter zurück vorbei vorher nachher inzwischen mittlerweile zugleich gleichzeitig zunächst "
           "anschließend außerdem ebenfalls ebenso genauso so zu sehr mehr kaum ganz völlig absolut recht "
           "überhaupt übrigens allerdings jedoch dennoch trotzdem deshalb deswegen daher folglich somit also "
           "nämlich offenbar anscheinend vermutlich wahrscheinlich hoffentlich leider zumindest wenigstens "
           "mindestens höchstens ungefähr circa zirka rund knapp annähernd gern gerne lieber am liebsten "
           "meist meistens manchmal gelegentlich häufig oftmals stets ständig dauernd immerzu nirgends nirgendwo "
           "überall irgendwo irgendwann irgendwie jemals bisher hin her herauf heraus herein herüber herum "
           "herunter hinab hinauf hinaus hinein hinüber hinunter hinten vorne oben unten links rechts drinnen "
           "draußen drüben drauf drin drum dran drüber ab an auf aus ein mit nach vor zu weg fort empor entgegen "
           "gegenüber umher umso je desto zudem ferner sodann alsdann einst ehemals früher später künftig "
           "neulich soeben gerade eben momentan derzeit zurzeit sonst anders sowieso ohnehin schließlich endlich "
           "plötzlich sofort gleich sogleich umgehend rasch schnell langsam teilweise größtenteils "
           "hauptsächlich vorwiegend überwiegend zumeist ohnedies quasi gewissermaßen sozusagen beinahe nahezu "
           "kurzerhand keinesfalls keineswegs allenfalls andernfalls ansonsten jedenfalls keinesfalls "
           "ebendort ebenso soweit soviel sowohl wie weshalb wieso warum weswegen wann wo wie wieviel").split()


def entries():
    seen = {}
    for part in (determiners, pronouns, conjunctions, prepositions, numerals):
        for w, f in part():
            seen.setdefault(w, set()).update(f)
    for w in ADVERBS:
        seen.setdefault(w, set()).add("r")
    for adj in UMLAUT_ADJECTIVES:
        comp, sup = umlaut_degrees(adj)
        seen.setdefault(comp, set()).update("J" + ADJ_ENDINGS)
        seen.setdefault(sup, set()).update("J" + ADJ_ENDINGS)
    return seen


if __name__ == "__main__":
    for w, f in sorted(entries().items()):
        print(w, "".join(sorted(f)))
