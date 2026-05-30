// Working German test — exercises the PlainGerman parser and basic linting
// via the public LintGroup API.

use harper_core::linting::{LintGroup, Linter};
use harper_core::parsers::{Markdown, MarkdownOptions, Parser, PlainGerman};
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
    use std::sync::Arc;

    let dict = Arc::new(curated_german_dictionary());
    let mut linter = LintGroup::new_curated(dict.clone(), Dialect::German);

    let text =
        std::fs::read_to_string("tests/test_sources/german_basic.md").expect("test file missing");
    let parser = Markdown::new(MarkdownOptions::default());
    let doc = Document::new(&text, &parser, &dict);

    // Simulate what document_state::generate_diagnostics does
    let temp = linter.config.clone();
    linter.config.fill_with_curated_for(Dialect::German);
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

#[test]
fn test_german_curated_config_disables_english_indefinite_article_rule() {
    let dict = curated_german_dictionary();
    let mut linter = LintGroup::new_curated(dict.clone(), Dialect::German);
    let text = "Die Übergabe hat unmittelbar an die neue Verwaltung zu erfolgen.";
    let document = Document::new(text, &PlainGerman, &dict);

    let temp = linter.config.clone();
    linter.config.fill_with_curated_for(Dialect::German);
    let lints_map = linter.organized_lints(&document);
    linter.config = temp;

    let all_lints: Vec<_> = lints_map.values().flat_map(|lints| lints.iter()).collect();

    assert!(
        all_lints
            .iter()
            .all(|lint| lint.message != "Incorrect indefinite article."),
        "German text should not trigger the English indefinite article rule: {all_lints:?}"
    );
    assert!(!linter.config.is_rule_enabled("AnA"));
}

fn lint_markdown_fixture(path: &str) -> Vec<String> {
    let dict = curated_german_dictionary();
    let mut linter = LintGroup::new_curated(dict.clone(), Dialect::German);
    let text = std::fs::read_to_string(path).expect("test file missing");
    let parser = Markdown::new(MarkdownOptions::default());
    let document = Document::new(&text, &parser, &dict);

    linter
        .lint(&document)
        .into_iter()
        .map(|lint| lint.message)
        .collect()
}

#[ignore = "German dictionary incomplete - words like Sicherungskonzept, Überblick need to be added"]
#[test]
fn test_german_storage_fixture_stays_clean() {
    let messages = lint_markdown_fixture("tests/test_sources/german_storage_guide.md");

    assert!(
        messages.is_empty(),
        "Storage guide should lint cleanly, got {messages:?}"
    );
}

#[ignore = "German dictionary incomplete - words like Produktbasiert, Frühlingsversion need to be added"]
#[test]
fn test_german_release_notes_fixture_stays_clean() {
    let messages = lint_markdown_fixture("tests/test_sources/german_release_notes.md");

    assert!(
        messages.is_empty(),
        "Release notes should lint cleanly, got {messages:?}"
    );
}

#[test]
fn test_german_support_fixture_surfaces_expected_issues() {
    let messages = lint_markdown_fixture("tests/test_sources/german_support_ticket.md");

    assert!(
        messages
            .iter()
            .any(|message| message.contains("Festplattenspeicer")),
        "Support fixture should flag the misspelled compound, got {messages:?}"
    );
    assert!(
        messages.len() >= 3,
        "Support fixture should surface several issues, got {messages:?}"
    );
}
