// Component-level workflow tests for German language support.
// These cover detection + parsing + linting combinations, while backend-level
// LSP open/change/command flows are tested in backend.rs.

use harper_core::GermanDialect;
use harper_core::language_detection::LanguageDetectionRegistry;
use harper_core::spell::{FstDictionary, curated_german_dictionary};
use harper_core::{Document, EnglishDialect, Language};

struct Dialect;
impl Dialect {
    const AMERICAN: Language = Language::English(EnglishDialect::American);
    const GERMAN: Language = Language::German(GermanDialect::Standard);
}

/// Test full workflow: open German file → auto-detect → lint → suggest corrections
#[test]
fn test_full_workflow_german_document() {
    let registry = LanguageDetectionRegistry::new();
    let dict = FstDictionary::curated(); // English dictionary for detection

    // Step 1: Auto-detect language
    let german_text = "der Hund spielt im Garten. das Auto ist schnell.";
    let detected = registry.detect_language(german_text, &dict, Dialect::AMERICAN);

    assert_eq!(detected, Dialect::GERMAN, "Should auto-detect German text");

    // Step 2: Parse document with correct parser
    let document = Document::new(
        german_text,
        &harper_core::parsers::PlainGerman,
        &curated_german_dictionary(),
    );

    // Step 3: Lint the document
    use harper_core::linting::{LintGroup, Linter};
    let mut linter = LintGroup::new_curated(dict, Dialect::GERMAN);
    let lints = linter.lint(&document);

    // Step 4: Verify suggestions are generated
    assert!(
        !lints.is_empty(),
        "Should detect capitalization errors in German text"
    );

    // Verify we get specific suggestions
    let capitalization_lints: Vec<_> = lints
        .iter()
        .filter(|l| l.message.contains("capital"))
        .collect();

    assert!(
        !capitalization_lints.is_empty(),
        "Should suggest capitalization fixes for 'der' and 'das'"
    );

    // Verify at least one lint has a suggestion
    let lints_with_suggestions: Vec<_> =
        lints.iter().filter(|l| !l.suggestions.is_empty()).collect();

    assert!(
        !lints_with_suggestions.is_empty(),
        "At least one lint should have correction suggestions"
    );
}

/// Test full workflow with German spelling errors
#[test]
fn test_full_workflow_german_spelling_errors() {
    let registry = LanguageDetectionRegistry::new();
    let dict = FstDictionary::curated(); // English dictionary for detection

    // German text with spelling errors
    let text = "Der Hunte ist im Gartens. dieser Satz ist klein.";

    // Auto-detect
    let detected = registry.detect_language(text, &dict, Dialect::AMERICAN);
    assert_eq!(detected, Dialect::GERMAN, "Should detect German");

    // Parse and lint
    let document = Document::new(
        text,
        &harper_core::parsers::PlainGerman,
        &curated_german_dictionary(),
    );

    use harper_core::linting::{LintGroup, Linter};
    let mut linter = LintGroup::new_curated(dict, Dialect::GERMAN);
    let lints = linter.lint(&document);

    // Should detect multiple errors
    assert!(
        lints.len() >= 2,
        "Should detect spelling errors: 'Hunte' and 'Gartens', got {} lints",
        lints.len()
    );

    // Verify we have suggestions for the misspellings
    let spelling_lints: Vec<_> = lints
        .iter()
        .filter(|l| l.message.contains("spelling") || l.message.contains("Spelling"))
        .collect();

    assert!(
        !spelling_lints.is_empty(),
        "Should detect spelling errors and provide suggestions"
    );
}

/// Test mixed-language document: German with English quotes
#[test]
fn test_mixed_language_german_english_quotes() {
    let registry = LanguageDetectionRegistry::new();
    let dict = FstDictionary::curated(); // English dictionary for detection

    // German text with English quote
    let text = "Der Autor sagt: \"The quick brown fox jumps over the lazy dog.\"";

    // Should detect one language (both are acceptable for mixed content)
    let detected = registry.detect_language(text, &dict, Dialect::AMERICAN);
    assert!(
        detected == Dialect::GERMAN || detected == Dialect::AMERICAN,
        "Should detect a language for mixed content, got {:?}",
        detected
    );

    // Parse and lint - should handle gracefully
    let document = Document::new(
        text,
        &harper_core::parsers::PlainGerman,
        &curated_german_dictionary(),
    );

    use harper_core::linting::{LintGroup, Linter};
    let mut linter = LintGroup::new_curated(dict, Dialect::GERMAN);

    // Should not crash on mixed content
    let lints = linter.lint(&document);
    assert!(
        lints.len() < 20,
        "Mixed language should not generate excessive lints"
    );
}

/// Test mixed-language document: English with German technical terms
#[test]
fn test_mixed_language_english_german_terms() {
    let registry = LanguageDetectionRegistry::new();
    let dict = FstDictionary::curated(); // English dictionary for detection

    // English text with German technical terms
    let text = "The Kindergarten is in Germany. The Doppelgänger effect is strange.";

    // Should detect one language (both are acceptable for mixed content)
    let detected = registry.detect_language(text, &dict, Dialect::AMERICAN);
    assert!(
        detected == Dialect::GERMAN || detected == Dialect::AMERICAN,
        "Should detect a language for mixed content, got {:?}",
        detected
    );

    // Should not crash or generate excessive lints
    let document = Document::new_curated(text, &harper_core::parsers::PlainEnglish);

    use harper_core::linting::{LintGroup, Linter};
    let mut linter = LintGroup::new_curated(dict, Dialect::AMERICAN);
    let lints = linter.lint(&document);

    assert!(
        lints.len() < 10,
        "Loanwords should not generate excessive lints"
    );
}

/// Test language detection with code-switching (mid-sentence language change)
#[test]
fn test_code_switching_mid_sentence() {
    let registry = LanguageDetectionRegistry::new();
    let dict = FstDictionary::curated(); // English dictionary for detection

    // Sentence starts in German, switches to English
    let text = "Das Auto ist fast wie the car in the movie.";

    // Detect primary language
    let detected = registry.detect_language(text, &dict, Dialect::AMERICAN);

    // Should pick one (either is acceptable for mixed content)
    assert!(
        detected == Dialect::GERMAN || detected == Dialect::AMERICAN,
        "Should detect a language, got {:?}",
        detected
    );

    // Should not crash on code-switching
    let document = match detected {
        Dialect::GERMAN => Document::new(
            text,
            &harper_core::parsers::PlainGerman,
            &curated_german_dictionary(),
        ),
        _ => Document::new_curated(text, &harper_core::parsers::PlainEnglish),
    };

    use harper_core::linting::{LintGroup, Linter};
    let mut linter = LintGroup::new_curated(
        match detected {
            Dialect::GERMAN => curated_german_dictionary(),
            _ => harper_core::spell::FstDictionary::curated(),
        },
        detected,
    );

    let lints = linter.lint(&document);
    // Should handle code-switching gracefully
    assert!(
        lints.len() < 50,
        "Code-switching should not cause explosion of lints"
    );
}

/// Test edge case: empty document
#[test]
fn test_empty_document_workflow() {
    let registry = LanguageDetectionRegistry::new();
    let dict = FstDictionary::curated(); // English dictionary for detection

    let text = "";

    // Should default to provided default dialect
    let detected = registry.detect_language(text, &dict, Dialect::AMERICAN);
    assert_eq!(
        detected,
        Dialect::AMERICAN,
        "Empty document should default to American English"
    );

    // Should handle empty document gracefully
    let document = Document::new_curated(text, &harper_core::parsers::PlainEnglish);

    use harper_core::linting::{LintGroup, Linter};
    let mut linter = LintGroup::new_curated(dict, Dialect::AMERICAN);
    let lints = linter.lint(&document);

    assert!(lints.is_empty(), "Empty document should have no lints");
}

/// Test edge case: very short text
#[test]
fn test_very_short_text_workflow() {
    let registry = LanguageDetectionRegistry::new();
    let dict = FstDictionary::curated(); // English dictionary for detection

    // Very short German text
    let text = "Hund";

    // Should default for very short text
    let detected = registry.detect_language(text, &dict, Dialect::AMERICAN);
    assert_eq!(
        detected,
        Dialect::AMERICAN,
        "Very short text should default to American English"
    );

    // Should handle short text gracefully
    let document = Document::new_curated(text, &harper_core::parsers::PlainEnglish);

    use harper_core::linting::{LintGroup, Linter};
    let mut linter = LintGroup::new_curated(dict, Dialect::AMERICAN);
    let lints = linter.lint(&document);

    // May have lints but should not crash
    assert!(lints.len() < 10, "Short text should have minimal lints");
}

/// Test performance: full workflow on realistic German paragraph
#[test]
fn test_full_workflow_performance() {
    let registry = LanguageDetectionRegistry::new();
    let dict = FstDictionary::curated(); // English dictionary for detection

    // Realistic German paragraph with some errors
    let text = "Der Hund spielt im Garten mit dem Ball. \
                die Katze schläft auf dem Sofa im Wohnzimmer. \
                das Auto ist sehr schnell und fährt auf der Straße. \
                Wir gehen heute ins Kino und essen danach im Restaurant.";

    let start = std::time::Instant::now();

    // Step 1: Detect
    let detected = registry.detect_language(text, &dict, Dialect::AMERICAN);

    // Step 2: Parse
    let document = Document::new(
        text,
        &harper_core::parsers::PlainGerman,
        &curated_german_dictionary(),
    );

    // Step 3: Lint
    use harper_core::linting::{LintGroup, Linter};
    let mut linter = LintGroup::new_curated(dict, Dialect::GERMAN);
    let lints = linter.lint(&document);

    let duration = start.elapsed();

    // Verify results
    assert_eq!(detected, Dialect::GERMAN, "Should detect German");
    assert!(
        lints.len() >= 2,
        "Should detect lowercase sentence starts: 'die' and 'das'"
    );

    // Verify performance (more lenient for debug builds and CI runners)
    assert!(
        duration.as_secs() < 3,
        "Full workflow should complete in < 3s, took {:?}",
        duration
    );
}
