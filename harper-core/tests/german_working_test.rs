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
    let mut linter = LintGroup::new_curated(dict.clone(), Dialect::German);

    let text = "Der Hund ist im Garten. Die Katze schläft auf dem Sofa.";
    let document = Document::new(text, &PlainGerman, &dict);
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
    let mut linter = LintGroup::new_curated(dict.clone(), Dialect::German);

    // lowercase sentence start + two misspellings
    let text = "Der Hund ist da. dieser Satz ist klein. Worrt und flasch.";
    let document = Document::new(text, &PlainGerman, &dict);
    let lints = linter.lint(&document);

    assert!(
        lints.len() >= 3,
        "Should find at least 3 issues (1 cap + 2 spelling), got {}",
        lints.len()
    );
}

/// Simulate exactly what harper-ls does: Markdown parser + German dict + organized_lints
#[test]
fn test_german_ls_simulation() {
    use harper_core::linting::LintGroup;
    use harper_core::parsers::{Markdown, MarkdownOptions};
    use harper_core::spell::curated_german_dictionary;
    use harper_core::{Dialect, Document};
    use std::sync::Arc;

    let dict = Arc::new(curated_german_dictionary());
    let mut linter = LintGroup::new_curated(dict.clone(), Dialect::German);

    let text =
        std::fs::read_to_string("tests/test_sources/german_basic.md").expect("test file missing");
    let parser = Markdown::new(MarkdownOptions::default());
    let doc = Document::new(&text, &parser, &dict);

    // Simulate what document_state::generate_diagnostics does
    let temp = linter.config.clone();
    linter.config.fill_with_curated();
    let lints_map = linter.organized_lints(&doc);
    linter.config = temp;

    let total: usize = lints_map.values().map(|v| v.len()).sum();
    println!("organized_lints found: {} total lints", total);
    for (origin, lints) in &lints_map {
        for lint in lints {
            println!("  [{}] {:?}: {}", origin, lint.lint_kind, lint.message);
        }
    }

    assert!(
        total >= 3,
        "Should find at least 3 issues (1 cap + 2 spelling), got {}",
        total
    );
}
