//! Verb forms the dictionary cannot describe one word at a time.

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
/// *nimm*. Spelling only — which of them is a real form is the dictionary's
/// to say, and for most verbs none is (*leb* → *lieb* is another verb).
///
/// A stem in *-t* is left out: its imperative does not have this shape
/// (*tritt*, *gilt*).
pub fn raised_stems(stem: &str) -> Vec<String> {
    let Some(at) = stem.rfind('e') else {
        return Vec::new();
    };
    let (head, tail) = (&stem[..at], &stem[at + 1..]);
    if stem.ends_with('t') || tail.chars().any(|c| "aeiouäöü".contains(c)) {
        return Vec::new();
    }
    let mut stems = vec![format!("{head}i{tail}"), format!("{head}ie{tail}")];
    // *nehm* → *nimm*: the length mark goes and the consonant doubles.
    let mut rest = tail.chars();
    if let (Some('h'), Some(consonant), None) = (rest.next(), rest.next(), rest.next()) {
        stems.push(format!("{head}i{consonant}{consonant}"));
    }
    stems
}

#[cfg(test)]
mod tests {
    use super::{has_zu_infix, looks_like_participle, raised_stems, without_zu_infix};

    #[test]
    fn the_e_is_raised_to_i_or_ie() {
        assert_eq!(raised_stems("geb"), ["gib", "gieb"]);
        assert_eq!(raised_stems("les"), ["lis", "lies"]);
        assert!(raised_stems("nehm").contains(&"nimm".to_string()));
        assert!(raised_stems("vergess").contains(&"vergiss".to_string()));
        assert!(raised_stems("tret").is_empty());
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
