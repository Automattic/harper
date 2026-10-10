//! Verb forms the dictionary cannot describe one word at a time.

use crate::language::german::spell::curated_german_dictionary;
use crate::language::german::spell::lexical_classes::STRONG_PRESENT_FORMS;
use crate::spell::Dictionary;

/// The conjunctions that open an infinitive group of their own: *um … zu*,
/// *ohne … zu*, *(an)statt … zu*. The *zu* in the group belongs to them, not to
/// a verb in front.
pub const INFINITIVE_GROUP_OPENERS: &[&str] = &["um", "ohne", "statt", "anstatt"];

/// The separable prefixes that take the *zu* of an infinitive inside the word:
/// *anzurufen*, *aufzuhören*. *kennen* is no prefix, but *kennenlernen* is
/// written as one word and puts its *zu* in the same place.
const SEPARABLE_PREFIXES: &[&str] = &[
    "an", "auf", "aus", "ab", "ein", "mit", "vor", "nach", "her", "hin", "weg", "zurück", "fest",
    "los", "vorbei", "teil", "dar", "bei", "zusammen", "fern", "frei", "heim", "kennen",
];

/// The separable verb a particle at the end of a clause and a finite verb
/// earlier in it make together: *aus* + *zahlt* → *auszahlt*, *zu* + *hört* →
/// *zuhört*, *kennen* + *lerne* → *kennenlerne*. The dictionary has to know
/// the joined form as a verb.
///
/// *zu* and *weiter* cannot be prefixes with a *zu* inside (*zuzuhören* is
/// written, but `zu_infix_parts` would read *zu* + *zuhören*), so they are only
/// admitted here.
pub fn joined_separable_verb(particle: &str, verb: &str) -> Option<String> {
    if !SEPARABLE_PREFIXES.contains(&particle) && !matches!(particle, "zu" | "weiter") {
        return None;
    }
    let joined = format!("{particle}{verb}");
    is_verb(&joined).then_some(joined)
}

/// Whether `word` is spelled like an infinitive with its *zu* inside, as a
/// separable verb writes it: *anzurufen*, *mitzunehmen*, *kennenzulernen*.
///
/// Only the spelling is checked — the caller still has to know that the word
/// is a verb. At least three letters must follow the *zu*, so that *anzug*
/// and *abzug* do not count.
pub fn has_zu_infix(word: &str) -> bool {
    without_zu_infix(word).is_some()
}

/// The infinitive `word` spells with its *zu* inside, without the *zu*:
/// *anzurufen* → *anrufen*, *kennenzulernen* → *kennenlernen*.
pub fn without_zu_infix(word: &str) -> Option<String> {
    zu_infix_parts(word).map(|(prefix, rest)| format!("{prefix}{rest}"))
}

/// The separable prefix in front of the *zu*, and what follows the *zu*:
/// *anzurufen* → (*an*, *rufen*).
pub fn zu_infix_parts(word: &str) -> Option<(&'static str, String)> {
    let lower = word.to_lowercase();
    SEPARABLE_PREFIXES.iter().find_map(|&prefix| {
        let rest = lower.strip_prefix(prefix)?.strip_prefix("zu")?;
        (rest.chars().count() >= 3).then(|| (prefix, rest.to_string()))
    })
}

/// Whether `word` is spelled like a past participle with its *ge*:
/// *gefordert*, *aufgefordert*, *eingeladen*, *gezwungen*.
///
/// The dictionary often knows such a form only as an adjective, which is
/// what it also is. The caller decides whether the shape is enough.
pub fn looks_like_participle(word: &str) -> bool {
    let lower = word.to_lowercase();
    let after_prefix = SEPARABLE_PREFIXES
        .iter()
        .find_map(|prefix| {
            lower
                .strip_prefix(prefix)
                .filter(|rest| rest.starts_with("ge"))
        })
        .unwrap_or(&lower);
    after_prefix.starts_with("ge")
        && after_prefix.chars().count() >= 5
        && (after_prefix.ends_with('t') || after_prefix.ends_with("en"))
}

/// The stems a strong verb's *e* is raised to in the second and third person
/// singular and the imperative: *geb* → *gib*, *les* → *lies*, *nehm* →
/// *nimm*, *tret* → *tritt*, *gelt* → *gilt*. Spelling only — which of them is
/// a real form is the dictionary's to say, and for most verbs none is (*leb*
/// → *lieb* is another verb). See [`raised_stem`].
pub fn raised_stems(stem: &str) -> Vec<String> {
    let Some(at) = stem.rfind('e') else {
        return Vec::new();
    };
    let (head, tail) = (&stem[..at], &stem[at + 1..]);
    if tail.chars().any(|c| "aeiouäöü".contains(c)) {
        return Vec::new();
    }
    let mut stems = vec![format!("{head}i{tail}"), format!("{head}ie{tail}")];
    let mut rest = tail.chars();
    match (rest.next(), rest.next(), rest.next()) {
        // *nehm* → *nimm*: the length mark goes and the consonant doubles.
        (Some('h'), Some(consonant), None) => {
            stems.push(format!("{head}i{consonant}{consonant}"));
        }
        // *tret* → *tritt*: the short vowel doubles the *t*.
        (Some('t'), None, None) => stems.push(format!("{head}itt")),
        _ => {}
    }
    stems
}

/// The third person singular on a raised or umlauted stem: *gib* → *gibt*,
/// *fähr* → *fährt*, and a stem in *-t* is its own third person, *tritt*,
/// *gilt*, *hält*.
pub fn third_person_of_raised(raised: &str) -> String {
    if raised.ends_with('t') {
        raised.to_string()
    } else {
        format!("{raised}t")
    }
}

/// The second person singular on a raised stem: *gib* → *gibst*, *lies* →
/// *liest*, *iss* → *isst*, *tritt* → *trittst*. A stem in a sibilant takes
/// a bare *-t*.
pub fn second_person_of_raised(raised: &str) -> String {
    if raised.ends_with(['s', 'ß', 'z', 'x']) {
        format!("{raised}t")
    } else {
        format!("{raised}st")
    }
}

/// The present-tense form of a strong verb spelled on the plain stem, and the
/// changed stem it should have had: *gebt* → (*gib*, `t`), *lest* → (*lies*,
/// `t`), *tretet* → (*tritt*, `t`), *nehmst* → (*nimm*, `st`), *fahrt* →
/// (*fähr*, `t`), *haltet* → (*hält*, `t`).
///
/// *gebt*, *lest* and *tretet* are real words — the second person plural, *ihr
/// gebt* — but nothing else; *er gebt* is wrong. *nehmst* is not a word at
/// all. The ending tells the caller which: `"t"` for the plural form, `"st"`
/// for the made-up second person singular.
pub fn plain_stem_present(word: &str) -> Option<(String, &'static str)> {
    let lower = word.to_lowercase();
    let changed = |stem: &str| raised_stem(stem).or_else(|| umlauted_stem(stem));
    // *gebt*, *lest*, *esst*, *fahrt*: the stem and a *-t*. *tretet*,
    // *haltet*, *ladet*: a stem in *-t* or *-d* takes *-et*.
    if let Some(stem) = lower
        .strip_suffix("et")
        .filter(|stem| stem.ends_with(['t', 'd']))
        && let Some(raised) = changed(stem)
    {
        return Some((raised, "t"));
    }
    if let Some(stem) = lower.strip_suffix('t')
        && let Some(raised) = changed(stem)
    {
        return Some((raised, "t"));
    }
    let stem = lower
        .strip_suffix("est")
        .filter(|stem| stem.ends_with(['t', 'd']))
        .or_else(|| lower.strip_suffix("st"))?;
    changed(stem).map(|raised| (raised, "st"))
}

fn is_verb(word: &str) -> bool {
    let chars: Vec<char> = word.chars().collect();
    curated_german_dictionary()
        .get_word_metadata(&chars)
        .is_some_and(|metadata| metadata.is_verb())
}

/// Is `stem` the stem of a verb with a strong past, so that a changed vowel
/// in the present is to be expected? The infinitive has to be a verb, and the
/// weak preterite must not be: *backte*, *fragte*, *erschreckte* and
/// *melkte* exist, so *er backt*, *er fragt*, *er erschreckt ihn* and *er
/// melkt* are correct beside the rarer *bäckt*, *frägt*, *erschrickt*,
/// *milkt*.
fn is_strong_infinitive_stem(stem: &str) -> bool {
    stem.chars().count() >= 2
        && is_verb(&format!("{stem}en"))
        && !is_verb(&format!("{stem}te"))
        // *-ete* only where the stem ends in *t* or *d* (*rettete*): *gebete*
        // is the plural of *Gebet*.
        && !(stem.ends_with(['t', 'd']) && is_verb(&format!("{stem}ete")))
}

/// The stems a strong verb's *a*, *au* or *o* is umlauted to in the second
/// and third person singular: *fahr* → *fähr*, *lauf* → *läuf*, *halt* →
/// *hält*, *stoß* → *stöß*. Unlike the *e/i* verbs, the imperative keeps the
/// plain vowel (*fahr!*, *lauf!*).
pub fn umlauted_stems(stem: &str) -> Vec<String> {
    let Some(at) = stem.rfind(['a', 'o']) else {
        return Vec::new();
    };
    let (head, tail) = (&stem[..at], &stem[at + 1..]);
    if tail.chars().any(|c| "aeiouäöü".contains(c) && c != 'u') {
        return Vec::new();
    }
    let umlaut = if stem[at..].starts_with('a') {
        "ä"
    } else {
        "ö"
    };
    vec![format!("{head}{umlaut}{tail}")]
}

/// The umlauted stem of the strong verb on `stem`, if it is one: *fahr* →
/// *fähr*, *halt* → *hält*; *zahl* → `None`, because *zählt* is *zählen*'s.
/// The same marker as for [`raised_stem`] decides.
pub fn umlauted_stem(stem: &str) -> Option<String> {
    if !is_strong_infinitive_stem(stem) {
        return None;
    }
    umlauted_stems(stem)
        .into_iter()
        .find(|umlauted| STRONG_PRESENT_FORMS.contains(&third_person_of_raised(umlauted)))
}

/// The raised stem of the strong verb on `stem`, if it is one: *geb* →
/// *gib*, *les* → *lies*, *tret* → *tritt*; *leb* → `None`.
///
/// The dictionary decides: the third person on the raised stem has to carry
/// the strong-present marker `%`, which `add_german_strong_imperatives.py`
/// writes from hunspell's morphology. *liebt* is a verb but *lieben*'s, and
/// *hiebt* one but *hauen*'s preterite, so *leb* and *heb* are not raised.
/// The infinitive on the plain stem has to be a verb with no weak preterite.
pub fn raised_stem(stem: &str) -> Option<String> {
    if !is_strong_infinitive_stem(stem) {
        return None;
    }
    raised_stems(stem)
        .into_iter()
        .find(|raised| STRONG_PRESENT_FORMS.contains(&third_person_of_raised(raised)))
}

#[cfg(test)]
mod tests {
    use super::{
        has_zu_infix, looks_like_participle, plain_stem_present, raised_stem, raised_stems,
        second_person_of_raised, without_zu_infix,
    };

    #[test]
    fn the_dictionary_decides_which_verbs_are_strong() {
        for (stem, raised) in [
            ("geb", "gib"),
            ("nehm", "nimm"),
            ("les", "lies"),
            ("ess", "iss"),
            ("vergess", "vergiss"),
            ("helf", "hilf"),
            ("sprech", "sprich"),
            ("tret", "tritt"),
        ] {
            assert_eq!(raised_stem(stem).as_deref(), Some(raised), "{stem}");
        }
        for stem in [
            "leb", "bet", "rett", "mach", "geh", "steh", "werd", "stell", "fehl",
        ] {
            assert_eq!(raised_stem(stem), None, "{stem}");
        }
    }

    #[test]
    fn a_present_form_on_the_plain_stem_is_found() {
        for (word, raised, ending) in [
            ("gebt", "gib", "t"),
            ("lest", "lies", "t"),
            ("esst", "iss", "t"),
            ("vergesst", "vergiss", "t"),
            ("tretet", "tritt", "t"),
            ("nehmst", "nimm", "st"),
            ("sprechst", "sprich", "st"),
            ("fahrt", "fähr", "t"),
            ("lauft", "läuf", "t"),
            ("haltet", "hält", "t"),
            ("schlaft", "schläf", "t"),
            ("tragt", "träg", "t"),
        ] {
            assert_eq!(
                plain_stem_present(word),
                Some((raised.to_string(), ending)),
                "{word}"
            );
        }
        for word in [
            "gibt", "liest", "isst", "tritt", "macht", "lebt", "betet", "rettet",
        ] {
            assert_eq!(plain_stem_present(word), None, "{word}");
        }
        assert_eq!(second_person_of_raised("gib"), "gibst");
        assert_eq!(second_person_of_raised("lies"), "liest");
        assert_eq!(second_person_of_raised("tritt"), "trittst");
    }

    #[test]
    fn the_e_is_raised_to_i_or_ie() {
        assert_eq!(raised_stems("geb"), ["gib", "gieb"]);
        assert_eq!(raised_stems("les"), ["lis", "lies"]);
        assert!(raised_stems("nehm").contains(&"nimm".to_string()));
        assert!(raised_stems("vergess").contains(&"vergiss".to_string()));
        assert!(raised_stems("tret").contains(&"tritt".to_string()));
        assert!(raised_stems("gelt").contains(&"gilt".to_string()));
        assert!(raised_stems("mach").is_empty());
    }

    #[test]
    fn the_zu_inside_a_separable_verb_is_found() {
        for word in ["anzurufen", "aufzuhören", "kennenzulernen", "Mitzunehmen"] {
            assert!(has_zu_infix(word), "{word}");
        }
        for word in ["anzug", "abzug", "zurufen", "aufzug", "lernen"] {
            assert!(!has_zu_infix(word), "{word}");
        }
        assert_eq!(without_zu_infix("anzurufen").as_deref(), Some("anrufen"));
        assert_eq!(
            without_zu_infix("kennenzulernen").as_deref(),
            Some("kennenlernen")
        );
    }

    #[test]
    fn a_participle_is_recognized_by_its_ge() {
        for word in ["gefordert", "aufgefordert", "eingeladen", "gezwungen"] {
            assert!(looks_like_participle(word), "{word}");
        }
        for word in ["gern", "genug", "geht", "gelb", "anrufen"] {
            assert!(!looks_like_participle(word), "{word}");
        }
    }
}
