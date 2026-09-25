//! The number of a noun form.
//!
//! German flag letters name an affix and a property at once, and a letter that
//! is both is applied as both — see the collision table in `README.md`. The five
//! plural letters `X`, `Y`, `a`, `b` and `E` were in that state, and the two
//! readings said opposite things. The affix marks the form it *builds*: `Frau`
//! plus `Y` gives `Frauen`, a plural. The property marked the entry it *sits
//! on*, so `Frau` was a plural too — and so was every other singular noun that
//! can form one, 67268 of 67269 of them.
//!
//! That is not a cosmetic mislabel. A noun claiming to be plural agrees with
//! `den`, whose dative plural reading is the one thing that makes
//! *"mit **den** Freund"* look acceptable. The whole class of case errors that
//! only the noun can reveal was invisible because of it.
//!
//! The plural marking now lives on the affix alone, and the affix marks its base
//! singular in return — for four of the five. `E` is left out on purpose; the
//! affix comment in `annotations.json` says why.

#[cfg(test)]
mod tests {
    use crate::language::german::spell::curated_german_dictionary;
    use crate::language::morphology::{MorphologyExt, NumberSet};
    use crate::spell::Dictionary;

    fn number(word: &str) -> NumberSet {
        curated_german_dictionary()
            .get_word_metadata_str(word)
            .unwrap_or_else(|| panic!("{word} is not in the German dictionary"))
            .noun_agreement()
            .number
    }

    /// The regression itself: a singular noun that can form a plural is not
    /// thereby a plural.
    #[test]
    fn a_noun_that_forms_a_plural_is_singular() {
        for word in [
            "Mann",
            "Frau",
            "Haus",
            "Auto",
            "Kind",
            "Freund",
            "Brief",
            "Tisch",
            "Bild",
            "Zeit",
            "Aufklärung",
        ] {
            assert_eq!(number(word), NumberSet::SINGULAR, "{word}");
        }
    }

    #[test]
    fn the_generated_plural_is_plural() {
        for word in [
            "Frauen",
            "Autos",
            "Kinder",
            "Freunde",
            "Briefe",
            "Bilder",
            "Aufklärungen",
            "Diagnosen",
        ] {
            assert!(
                number(word).contains(NumberSet::PLURAL),
                "{word}: {:?}",
                number(word)
            );
        }
    }

    /// `den` is accusative masculine singular or dative plural. Only the noun
    /// can say which, and it can only say so if its own number is right.
    #[test]
    fn a_singular_noun_rules_out_the_dative_plural() {
        let freund = number("Freund");
        assert!(!freund.contains(NumberSet::PLURAL));
        assert!(freund.contains(NumberSet::SINGULAR));
    }

    /// An `-er` or `-el` noun has one form for the nominative singular and the
    /// nominative plural — *der Lehrer*, *die Lehrer* — so neither answer alone
    /// is right and the set holds both. That constrains nothing in an agreement
    /// check, which is the point: claiming either would invent an error.
    #[test]
    fn a_noun_with_one_form_for_both_numbers_holds_both() {
        for word in ["Lehrer", "Bäcker", "Körper"] {
            assert_eq!(
                number(word),
                NumberSet::SINGULAR | NumberSet::PLURAL,
                "{word}"
            );
        }
    }

    /// The other half of the same flag: an entry that is itself a plural and
    /// only adds the dative `-n`. It shares the flag with the line above, so
    /// the dictionary cannot tell the two apart and neither reading is dropped.
    #[test]
    fn a_plural_headword_that_takes_the_dative_n_is_not_forced_singular() {
        for word in ["Befunde", "Bereiche", "Ergebnisse"] {
            assert!(
                number(word).contains(NumberSet::PLURAL),
                "{word}: {:?}",
                number(word)
            );
        }
    }

    /// A form that is a real plural headword and a compound element at once
    /// keeps both readings rather than losing the plural. Without the union in
    /// `Agreement::or` these would come out singular and the rule would fire on
    /// *"mit den Flaschen"*.
    #[test]
    fn an_ambiguous_form_keeps_both_readings() {
        for word in ["Eichen", "Flaschen", "Branchen", "Wochen"] {
            assert_eq!(
                number(word),
                NumberSet::SINGULAR | NumberSet::PLURAL,
                "{word}"
            );
        }
    }
}
