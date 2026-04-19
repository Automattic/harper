// Comprehensive German MVP Test
// Tests the minimum viable product for German language support in Harper
// Uses the public LintGroup API with German dialect.

use harper_core::linting::{LintGroup, Linter};
use harper_core::parsers::{Parser, PlainGerman};
use harper_core::spell::curated_german_dictionary;
use harper_core::{Dialect, Document};

fn german_lint_group() -> LintGroup {
    let dict = curated_german_dictionary();
    LintGroup::new_curated(dict, Dialect::German)
}

/// Test 1: German parser functionality
#[test]
fn test_german_parser_basic() {
    let parser = PlainGerman;
    let text = "Der Hund ist im Garten.";
    let chars: Vec<char> = text.chars().collect();
    let tokens = parser.parse(&chars);

    assert!(!tokens.is_empty(), "Parser should produce tokens");
    let word_count = tokens.iter().filter(|t| t.kind.is_word()).count();
    assert_eq!(word_count, 5, "Should parse 5 German words");
}

/// Test 2: German special characters
#[test]
fn test_german_special_characters() {
    let parser = PlainGerman;
    let text = "Äpfel, Ökonomie, Größe, Überfluss";
    let chars: Vec<char> = text.chars().collect();
    let tokens = parser.parse(&chars);

    assert!(
        !tokens.is_empty(),
        "Should handle German special characters"
    );
}

/// Test 3: Detects lowercase sentence start in German
#[test]
fn test_german_sentence_capitalization() {
    let mut linter = german_lint_group();

    let text = "der Hund ist im Garten. das Auto ist schnell.";
    let document = Document::new_plain_english_curated(text);
    let lints = linter.lint(&document);

    let cap_lints: Vec<_> = lints
        .iter()
        .filter(|l| l.message.contains("capital") || l.message.contains("Capital"))
        .collect();

    assert!(
        cap_lints.len() >= 2,
        "Should detect at least 2 lowercase sentence starts, got {}",
        cap_lints.len()
    );
}

/// Test 4: German spell check detects misspellings
#[test]
fn test_german_spell_check() {
    let mut linter = german_lint_group();

    let text = "Der Hunte ist im Gartens.";
    let document = Document::new_plain_english_curated(text);
    let lints = linter.lint(&document);

    let spelling_lints: Vec<_> = lints
        .iter()
        .filter(|l| l.message.contains("spelling") || l.message.contains("Spelling"))
        .collect();

    assert!(
        !spelling_lints.is_empty(),
        "Should detect spelling errors in German text"
    );
}

/// Test 5: Correct German text produces few issues
#[test]
fn test_german_proper_grammar() {
    let mut linter = german_lint_group();

    let proper_text =
        "Der Hund ist im Garten. Das Auto ist schnell. Die Katze schläft auf dem Sofa.";
    let document = Document::new_plain_english_curated(proper_text);
    let lints = linter.lint(&document);

    assert!(
        lints.len() <= 2,
        "Correct German text should have at most 2 issues, got {}",
        lints.len()
    );
}

/// Test 6: Combined check on text with intentional errors
#[test]
fn test_german_mvp_comprehensive() {
    let mut linter = german_lint_group();

    // "dieser" starts a sentence lowercase, "Worrt" and "flasch" are misspelled
    let test_text = "Der Hund spielt im Garten. dieser Satz beginnt klein. Worrt ist flasch.";
    let document = Document::new_plain_english_curated(test_text);
    let lints = linter.lint(&document);

    assert!(
        lints.len() >= 3,
        "Should find at least 3 issues (1 capitalization + 2 spelling), got {}",
        lints.len()
    );
}

/// Test 7: Performance — lint a typical German paragraph in < 500ms
#[test]
fn test_german_performance() {
    let mut linter = german_lint_group();

    let text = "Der Hund ist im Garten. Die Katze schläft auf dem Sofa. \
                Das Auto ist sehr schnell. Die Kinder spielen im Park.";
    let document = Document::new_plain_english_curated(text);

    let start = std::time::Instant::now();
    let _lints = linter.lint(&document);
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 500,
        "Linting should finish in < 500ms, took {:?}",
        duration
    );
}

/// Test 8: German compound words should not be false positives
#[test]
fn test_german_compound_words() {
    let mut linter = german_lint_group();

    let text = "Das Gartenhaus ist groß. Das Haus ist klein.";
    let document = Document::new_plain_english_curated(text);
    let lints = linter.lint(&document);

    // "Haus" and "Gartenhaus" should both be recognized
    // Allow a small number of lints but not for these words
    for lint in &lints {
        assert!(
            !lint.message.contains("Haus"),
            "Should not flag 'Haus' as misspelled"
        );
    }
}
