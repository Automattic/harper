//! Tests for German compound checking.
//!
//! These live inside the German module rather than in `harper-core/tests/`
//! because they exercise crate-internal types (`AnnotatedWord`, `CompoundChecker`).
//! Keeping them here means `spell::rune` and `spell::word_map` stay `pub(crate)`
//! instead of being widened to public API just to support German.

#[cfg(test)]
mod tests {
    use crate::language::german::spell::compound_checker::CompoundChecker;
    use crate::spell::Dictionary;
    use crate::spell::rune::word_list::AnnotatedWord;

    fn annotated(letters: &str, annotations: &[char]) -> AnnotatedWord {
        AnnotatedWord {
            letters: letters.chars().collect(),
            annotations: annotations.to_vec(),
        }
    }

    #[test]
    fn compound_checker_creation() {
        let words = vec![
            annotated("schuh", &['N', 'X', 'h']),
            annotated("hersteller", &['N', 'h']),
        ];

        let checker = CompoundChecker::new(&words);
        assert!(checker.compound_word_count() > 0);
    }

    #[test]
    fn simple_compound_detection() {
        let words = vec![
            annotated("schuh", &['N', 'X', 'h']),
            annotated("hersteller", &['N', 'h']),
        ];

        let checker = CompoundChecker::new(&words);
        let schuhhersteller: Vec<char> = "schuhhersteller".chars().collect();
        assert!(checker.is_compound_word(&schuhhersteller));
    }

    #[test]
    fn compound_with_interfix() {
        let words = vec![
            annotated("arbeit", &['N', 'i']),
            annotated("geber", &['N', 'h']),
        ];

        let checker = CompoundChecker::new(&words);
        let arbeitsgeber: Vec<char> = "arbeitsgeber".chars().collect();
        assert!(checker.is_compound_word(&arbeitsgeber));
    }

    #[test]
    fn base_german_dictionary_contains_base_words() {
        let dict = crate::language::german::spell::base_german_dictionary();
        assert!(dict.contains_word(&"schuh".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn compound_aware_dictionary_is_functional() {
        let dict = crate::language::german::spell::compound_aware_german_dictionary();

        assert!(dict.contains_word(&"haus".chars().collect::<Vec<_>>()));
        assert!(dict.word_count() > 0);
    }

    #[test]
    fn compound_aware_dict_recognizes_base_and_compound() {
        let dict = crate::language::german::spell::compound_aware_german_dictionary();

        assert!(
            dict.contains_word_str("farbe"),
            "farbe should be in base dict"
        );
        assert!(
            dict.contains_word_str("wunsch"),
            "wunsch should be in base dict"
        );
        assert!(
            dict.contains_word_str("farbwunsch"),
            "farbwunsch should be recognized as compound"
        );
    }

    #[test]
    fn dictionary_level_productive_compounds() {
        let dict = crate::language::german::spell::compound_aware_german_dictionary();

        // Valid German compounds whose parts are plain dictionary words. The old
        // flag-based engine only accepted parts that carried compound-formation
        // annotations, so these were dictionary-level coverage gaps (the runtime
        // linter accepted them via its fallback). They must now be recognized by
        // the dictionary itself, so `get_word_metadata` and the coverage tool see
        // them too.
        for word in [
            "Interaktionsgeschehen",
            "Inspektionsaufgabe",
            "Differenzenquotient",
            "Eischnee",
            "Hühnerei",
        ] {
            assert!(
                dict.contains_word_str(word),
                "{word} should be recognized as a German compound"
            );
        }

        // Junk and misspelled compounds must remain rejected.
        for word in ["xyzabc", "Gartenhous", "Arbeitsplaz"] {
            assert!(
                !dict.contains_word_str(word),
                "{word} must not be accepted as a compound"
            );
        }
    }
}
