//! Integration tests for German compound checking implementation

use harper_core::spell::Dictionary;

#[cfg(all(feature = "de", feature = "multilingual"))]
#[test]
fn test_compound_checker_creation() {
    use harper_core::language::german::spell::compound_checker::CompoundChecker;
    use harper_core::spell::rune::word_list::AnnotatedWord;

    let words = vec![
        AnnotatedWord {
            letters: "schuh".chars().collect(),
            annotations: vec!['N', 'X', 'h'],
        },
        AnnotatedWord {
            letters: "hersteller".chars().collect(),
            annotations: vec!['N', 'h'],
        },
    ];

    let checker = CompoundChecker::new(&words);
    assert!(checker.compound_word_count() > 0);
}

#[cfg(feature = "de")]
#[test]
fn test_simple_compound_detection() {
    use harper_core::language::german::spell::compound_checker::CompoundChecker;
    use harper_core::spell::rune::word_list::AnnotatedWord;

    let words = vec![
        AnnotatedWord {
            letters: "schuh".chars().collect(),
            annotations: vec!['N', 'X', 'h'],
        },
        AnnotatedWord {
            letters: "hersteller".chars().collect(),
            annotations: vec!['N', 'h'],
        },
    ];

    let checker = CompoundChecker::new(&words);
    let schuhhersteller: Vec<char> = "schuhhersteller".chars().collect();
    assert!(checker.is_compound_word(&schuhhersteller));
}

#[cfg(feature = "de")]
#[test]
fn test_compound_with_interfix() {
    use harper_core::language::german::spell::compound_checker::CompoundChecker;
    use harper_core::spell::rune::word_list::AnnotatedWord;

    let words = vec![
        AnnotatedWord {
            letters: "arbeit".chars().collect(),
            annotations: vec!['N', 'i'],
        },
        AnnotatedWord {
            letters: "geber".chars().collect(),
            annotations: vec!['N', 'h'],
        },
    ];

    let checker = CompoundChecker::new(&words);
    let arbeitsgeber: Vec<char> = "arbeitsgeber".chars().collect();
    assert!(checker.is_compound_word(&arbeitsgeber));
}

#[cfg(feature = "de")]
#[test]
fn test_base_german_dictionary() {
    use harper_core::language::german::spell::base_german_dictionary;

    let dict = base_german_dictionary();
    assert!(dict.contains_word(&"schuh".chars().collect::<Vec<_>>()));
}

#[cfg(all(feature = "de", feature = "multilingual"))]
#[test]
fn test_compound_aware_dictionary() {
    use harper_core::language::german::spell::compound_aware_german_dictionary;

    let dict = compound_aware_german_dictionary();

    // Base words should be found
    assert!(dict.contains_word(&"haus".chars().collect::<Vec<_>>()));

    // Test that compound-aware dictionary is functional
    assert!(dict.word_count() > 0);
}

#[cfg(all(feature = "de", feature = "multilingual"))]
#[test]
fn test_compound_aware_dict_directly() {
    use harper_core::language::german::spell::compound_aware_german_dictionary;

    let dict = compound_aware_german_dictionary();

    // Test base words - these should work
    assert!(
        dict.contains_word_str("farbe"),
        "farbe should be in base dict"
    );
    assert!(
        dict.contains_word_str("wunsch"),
        "wunsch should be in base dict"
    );

    // Test compounds - these should work with lazy compound checking
    assert!(
        dict.contains_word_str("farbwunsch"),
        "farbwunsch should be recognized as compound"
    );
}
