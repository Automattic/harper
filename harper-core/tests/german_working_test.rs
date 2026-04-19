// Working German test — exercises the PlainGerman parser and basic linting
// via the public LintGroup API.

use harper_core::linting::{LintGroup, Linter};
use harper_core::parsers::{Parser, PlainGerman};
use harper_core::spell::curated_german_dictionary;
use harper_core::{Dialect, Document};

/// German parser handles special characters (umlauts and ß)
#[test]
fn test_german_parser_special_chars() {
    let parser = PlainGerman;
    let text = "Der Hund ist im Garten. Äpfel und Ökonomie.";

    let chars: Vec<char> = text.chars().collect();
    let tokens = parser.parse(&chars);

    assert!(!tokens.is_empty(), "Parser should produce tokens");
    assert!(
        tokens.iter().filter(|t| t.kind.is_word()).count() >= 8,
        "Should parse German words correctly"
    );
}

/// Parser handles several umlaut words
#[test]
fn test_german_umlauts() {
    let parser = PlainGerman;

    for word in &["Äpfel", "Öl", "Übung", "Straße", "Größe", "Schönheit"] {
        let chars: Vec<char> = word.chars().collect();
        let tokens = parser.parse(&chars);
        assert!(!tokens.is_empty(), "Should handle '{word}'");
    }
}

/// Parser performance on a typical sentence
#[test]
fn test_german_parser_performance() {
    let parser = PlainGerman;
    let text = "Der Hund läuft schnell durch den großen Garten. \
                Die Katze schläft gemütlich auf dem weichen Sofa.";

    let start = std::time::Instant::now();
    let chars: Vec<char> = text.chars().collect();
    let tokens = parser.parse(&chars);
    let duration = start.elapsed();

    assert!(duration.as_millis() < 100, "Parsing should be under 100ms");
    assert!(!tokens.is_empty(), "Should parse text successfully");
    let _ = tokens;
}

/// Real-world German sentences parse without panic
#[test]
fn test_german_real_world() {
    let parser = PlainGerman;

    let examples = [
        "Guten Tag! Wie geht es Ihnen?",
        "Ich spreche Deutsch.",
        "Das Wetter ist schön heute.",
        "Wo ist der Bahnhof?",
        "Ich hätte gerne ein Bier.",
    ];

    for text in &examples {
        let chars: Vec<char> = text.chars().collect();
        let tokens = parser.parse(&chars);
        assert!(!tokens.is_empty(), "Should parse '{text}'");
    }
}

/// LintGroup with German dialect flags known German words as correct
#[test]
fn test_german_lint_correct_text() {
    let dict = curated_german_dictionary();
    let mut linter = LintGroup::new_curated(dict, Dialect::German);

    let text = "Der Hund ist im Garten. Die Katze schläft auf dem Sofa.";
    let document = Document::new_plain_english_curated(text);
    let lints = linter.lint(&document);

    assert!(
        lints.len() <= 2,
        "Correct German text should produce at most 2 lints, got {}",
        lints.len()
    );
}

/// LintGroup detects intentional errors in German text
#[test]
fn test_german_lint_errors() {
    let dict = curated_german_dictionary();
    let mut linter = LintGroup::new_curated(dict, Dialect::German);

    // lowercase sentence start + two misspellings
    let text = "Der Hund ist da. dieser Satz ist klein. Worrt und flasch.";
    let document = Document::new_plain_english_curated(text);
    let lints = linter.lint(&document);

    assert!(
        lints.len() >= 3,
        "Should find at least 3 issues (1 cap + 2 spelling), got {}",
        lints.len()
    );
}
