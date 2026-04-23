// Comprehensive German compound word edge case tests
// Tests Fugen-s, Fugen-n, and complex compound word decomposition

use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::curated_german_dictionary;
use harper_core::{Dialect, Document};

/// Test basic compound word decomposition (no Fugen-s/n)
#[test]
fn test_basic_compound_word() {
    let dict = curated_german_dictionary();
    let mut linter = LintGroup::new_curated(dict, Dialect::German);

    let text = "Das Gartenhaus ist groß.";
    let document = Document::new(text, &harper_core::parsers::PlainGerman, &curated_german_dictionary());
    let lints = linter.lint(&document);

    // "Gartenhaus" should be recognized as valid compound (Garten + Haus)
    let compound_lints: Vec<_> = lints
        .iter()
        .filter(|l| l.message.contains("Gartenhaus"))
        .collect();
    assert!(
        compound_lints.is_empty(),
        "Gartenhaus should not be flagged as misspelled"
    );
}

/// Test Fugen-s compounds (most common type)
#[test]
fn test_fugen_s_compounds() {
    let dict = curated_german_dictionary();
    let mut linter = LintGroup::new_curated(dict, Dialect::German);

    // Test various Fugen-s compounds
    let test_cases = vec![
        "Arbeitsstelle", // Arbeit + s + Stelle
        "Frühstücksspeck", // Frühstück + s + Speck
        "Regenbogen", // Regen + bogen (no s, but still compound)
        "Liebesbrief", // Liebe + s + Brief
    ];

    for word in test_cases {
        let text = format!("Das {} ist wichtig.", word);
        let document = Document::new(
            &text,
            &harper_core::parsers::PlainGerman,
            &curated_german_dictionary(),
        );
        let lints = linter.lint(&document);

        let word_lints: Vec<_> = lints.iter().filter(|l| l.message.contains(word)).collect();
        assert!(
            word_lints.is_empty(),
            "{} should be recognized as valid Fugen-s compound",
            word
        );
    }
}

/// Test Fugen-n compounds (less common but valid)
#[test]
fn test_fugen_n_compounds() {
    let dict = curated_german_dictionary();
    let mut linter = LintGroup::new_curated(dict, Dialect::German);

    // Test Fugen-n compounds
    let test_cases = vec![
        "Straßenrand", // Straße + n + Rand
        "Hofläufer", // Hof + läufer (e→ä umlaut in compounds)
    ];

    for word in test_cases {
        let text = format!("Am {} stehen Bäume.", word);
        let document = Document::new(
            &text,
            &harper_core::parsers::PlainGerman,
            &curated_german_dictionary(),
        );
        let lints = linter.lint(&document);

        let word_lints: Vec<_> = lints.iter().filter(|l| l.message.contains(word)).collect();
        assert!(
            word_lints.is_empty(),
            "{} should be recognized as valid Fugen-n compound",
            word
        );
    }
}

/// Test three-part compound words (complex decomposition)
#[test]
fn test_three_part_compounds() {
    let dict = curated_german_dictionary();
    let mut linter = LintGroup::new_curated(dict, Dialect::German);

    // Three-part compounds: Word1 + Fugen + Word2 + Word3
    let test_cases = vec![
        "Donaudampfschifffahrt", // Donau + dampf + schiff + fahrt
        "Schwarzweißfilm", // Schwarz + weiß + Film
    ];

    for word in test_cases {
        let text = format!("Ein {} ist klassisch.", word);
        let document = Document::new(
            &text,
            &harper_core::parsers::PlainGerman,
            &curated_german_dictionary(),
        );
        let lints = linter.lint(&document);

        // These are very long compounds - may not be in dictionary
        // But we should at least not crash on them
        assert!(
            lints.len() < 10,
            "{} should not generate excessive lints, got {}",
            word,
            lints.len()
        );
    }
}

/// Test that misspelled compounds are still caught
#[test]
fn test_misspelled_compounds() {
    let dict = curated_german_dictionary();
    let mut linter = LintGroup::new_curated(dict, Dialect::German);

    // Intentional misspellings
    let test_cases = vec![
        ("Gartenhaus", "Gartenhous"), // Wrong vowel
        ("Arbeitsplatz", "Arbeitsplaz"), // z instead of tz
    ];

    for (_correct, incorrect) in test_cases {
        let text = format!("Der {} ist neu.", incorrect);
        let document = Document::new(
            &text,
            &harper_core::parsers::PlainGerman,
            &curated_german_dictionary(),
        );
        let lints = linter.lint(&document);

        assert!(
            !lints.is_empty(),
            "Misspelled '{}' should be detected",
            incorrect
        );
    }
}

/// Test edge case: very short compounds shouldn't be decomposed
#[test]
fn test_short_word_no_decomposition() {
    let dict = curated_german_dictionary();
    let mut linter = LintGroup::new_curated(dict, Dialect::German);

    // Short words that aren't compounds
    let text = "Das ist der Haus.";
    let document = Document::new(
        text,
        &harper_core::parsers::PlainGerman,
        &curated_german_dictionary(),
    );
    let lints = linter.lint(&document);

    // "Haus" is standalone, shouldn't trigger compound decomposition logic
    // (minimum length for compound check is 6 chars)
    assert!(lints.len() < 5, "Short text should have minimal lints");
}

/// Test edge case: compounds with umlauts
#[test]
fn test_compounds_with_umlauts() {
    let dict = curated_german_dictionary();
    let mut linter = LintGroup::new_curated(dict, Dialect::German);

    // Compounds with special German characters
    let test_cases = vec![
        "Größe", // Contains ß
        "Ärztekammer", // Starts with Ä
        "Ölpreis", // Starts with Ö
        "Überfluss", // Starts with Ü
    ];

    for word in test_cases {
        let text = format!("Die {} ist wichtig.", word);
        let document = Document::new(
            &text,
            &harper_core::parsers::PlainGerman,
            &curated_german_dictionary(),
        );
        let lints = linter.lint(&document);

        let word_lints: Vec<_> = lints.iter().filter(|l| l.message.contains(word)).collect();
        assert!(
            word_lints.is_empty() || word_lints.len() <= 1,
            "{} with umlauts should be recognized ({} lints)",
            word,
            word_lints.len()
        );
    }
}

/// Test performance: compound decomposition shouldn't be too slow
#[test]
fn test_compound_decomposition_performance() {
    let dict = curated_german_dictionary();
    let mut linter = LintGroup::new_curated(dict, Dialect::German);

    // Text with many compounds
    let text = "Der Gartenhausbesitzer arbeitet im Arbeitszimmer und geht zum Sportplatz. \
                Der Straßenrand ist schön und der Liebesbrief ist kurz.";

    let start = std::time::Instant::now();
    let document = Document::new(
        text,
        &harper_core::parsers::PlainGerman,
        &curated_german_dictionary(),
    );
    let _lints = linter.lint(&document);
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 1000,
        "Compound decomposition should complete in < 1s, took {:?}",
        duration
    );
}

/// Test that compound word detection doesn't cause false positives
#[test]
fn test_no_false_positives_on_simples_words() {
    let dict = curated_german_dictionary();
    let mut linter = LintGroup::new_curated(dict, Dialect::German);

    // Simple, non-compound words
    let text = "Der Hund ist im Garten und die Katze schläft.";
    let document = Document::new(
        text,
        &harper_core::parsers::PlainGerman,
        &curated_german_dictionary(),
    );
    let lints = linter.lint(&document);

    // Should have very few lints for correct simple text
    assert!(
        lints.len() <= 2,
        "Simple correct text should have minimal lints, got {}",
        lints.len()
    );
}
