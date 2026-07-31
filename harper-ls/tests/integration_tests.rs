// Component-level workflow tests for multi-language support.
// These cover detection + parsing + linting combinations, while backend-level
// LSP open/change/command flows are tested in backend.rs.

use harper_core::language::registry::{detect_language, new_curated_for_language};
use harper_core::spell::FstDictionary;
use harper_core::{Document, Dialect, Language};

/// Test edge case: empty document
#[test]
fn test_empty_document_workflow() {
    let dict = FstDictionary::curated(); // English dictionary for detection

    let text = "";

    // Should default to provided default dialect
    let detected = detect_language(text, &dict, Language::English(Dialect::American));
    assert_eq!(
        detected,
        Language::English(Dialect::American),
        "Empty document should default to American English"
    );

    // Should handle empty document gracefully
    let document = Document::new_curated(text, &harper_core::parsers::PlainEnglish);

    use harper_core::linting::Linter;
    let mut linter = new_curated_for_language(dict, Language::English(Dialect::American));
    let lints = linter.lint(&document);

    assert!(lints.is_empty(), "Empty document should have no lints");
}

/// Test edge case: very short text
#[test]
fn test_very_short_text_workflow() {
    let dict = FstDictionary::curated(); // English dictionary for detection

    // Very short German text
    let text = "Hund";

    // Should default for very short text
    let detected = detect_language(text, &dict, Language::English(Dialect::American));
    assert_eq!(
        detected,
        Language::English(Dialect::American),
        "Very short text should default to American English"
    );

    // Should handle short text gracefully
    let document = Document::new_curated(text, &harper_core::parsers::PlainEnglish);

    use harper_core::linting::Linter;
    let mut linter = new_curated_for_language(dict, Language::English(Dialect::American));
    let lints = linter.lint(&document);

    // May have lints but should not crash
    assert!(lints.len() < 10, "Short text should have minimal lints");
}

// German-specific tests - only compiled when the "de" feature is enabled
#[cfg(feature = "de")]
mod german_tests {
    use super::*;
    use harper_core::language::languages::LanguageFamily;
    use harper_core::language::registry::{dictionary, parser_for_prose};
    use harper_core::parsers::MarkdownOptions;

    /// Test full workflow: open German file → auto-detect → lint → suggest corrections
    #[test]
    fn test_full_workflow_german_document() {
        let dict = FstDictionary::curated(); // English dictionary for detection

        // Step 1: Auto-detect language
        let german_text = "der Hund spielt im Garten. das Auto ist schnell.";
        let default_lang = Language::English(Dialect::American);
        let detected = detect_language(german_text, &dict, default_lang);

        // Step 2: Get parser and dictionary for detected language
        let parser = parser_for_prose("plaintext", detected, MarkdownOptions::default())
            .expect("Should get parser for detected language");
        let language_dict = dictionary(detected);

        // Parse document with correct parser
        let document = Document::new(german_text, &parser, &language_dict);

        // Step 3: Lint the document
        use harper_core::linting::Linter;
        let mut linter = new_curated_for_language(dict, detected);
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
        let dict = FstDictionary::curated(); // English dictionary for detection

        // German text with spelling errors
        let text = "Der Hunte ist im Gartens. dieser Satz ist klein.";

        // Auto-detect
        let default_lang = Language::English(Dialect::American);
        let detected = detect_language(text, &dict, default_lang);

        // Parse and lint
        let parser = parser_for_prose("plaintext", detected, MarkdownOptions::default())
            .expect("Should get parser for detected language");
        let language_dict = dictionary(detected);
        let document = Document::new(text, &parser, &language_dict);

        use harper_core::linting::Linter;
        let mut linter = new_curated_for_language(dict, detected);
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
        let dict = FstDictionary::curated(); // English dictionary for detection

        // German text with English quote
        let text = "Der Autor sagt: \"The quick brown fox jumps over the lazy dog.\"";

        // Should detect one language (both are acceptable for mixed content)
        let default_lang = Language::English(Dialect::American);
        let detected = detect_language(text, &dict, default_lang);

        assert!(
            detected.family() == LanguageFamily::German
                || detected == Language::English(Dialect::American),
            "Should detect a language for mixed content, got {:?}",
            detected
        );

        // Parse and lint - should handle gracefully
        let parser = parser_for_prose("plaintext", detected, MarkdownOptions::default())
            .expect("Should get parser for detected language");
        let language_dict = dictionary(detected);
        let document = Document::new(text, &parser, &language_dict);

        use harper_core::linting::Linter;
        let mut linter = new_curated_for_language(dict, detected);

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
        let dict = FstDictionary::curated(); // English dictionary for detection

        // English text with German technical terms
        let text = "The Kindergarten is in Germany. The Doppelgänger effect is strange.";

        // Should detect one language (both are acceptable for mixed content)
        let default_lang = Language::English(Dialect::American);
        let detected = detect_language(text, &dict, default_lang);

        assert!(
            detected.family() == LanguageFamily::German
                || detected == Language::English(Dialect::American),
            "Should detect a language for mixed content, got {:?}",
            detected
        );

        // Should not crash or generate excessive lints
        let parser = parser_for_prose("plaintext", detected, MarkdownOptions::default())
            .expect("Should get parser for detected language");
        let language_dict = dictionary(detected);
        let document = Document::new(text, &parser, &language_dict);

        use harper_core::linting::Linter;
        let mut linter = new_curated_for_language(dict, detected);
        let lints = linter.lint(&document);

        assert!(
            lints.len() < 10,
            "Loanwords should not generate excessive lints"
        );
    }

    /// Test language detection with code-switching (mid-sentence language change)
    #[test]
    fn test_code_switching_mid_sentence() {
        let dict = FstDictionary::curated(); // English dictionary for detection

        // Sentence starts in German, switches to English
        let text = "Das Auto ist fast wie the car in the movie.";

        // Detect primary language
        let default_lang = Language::English(Dialect::American);
        let detected = detect_language(text, &dict, default_lang);

        // Should pick one (either is acceptable for mixed content)
        assert!(
            detected.family() == LanguageFamily::German
                || detected == Language::English(Dialect::American),
            "Should detect a language, got {:?}",
            detected
        );

        // Should not crash on code-switching
        let parser = parser_for_prose("plaintext", detected, MarkdownOptions::default())
            .expect("Should get parser for detected language");
        let language_dict = dictionary(detected);
        let document = Document::new(text, &parser, &language_dict);

        use harper_core::linting::Linter;
        let mut linter = new_curated_for_language(dict, detected);

        let lints = linter.lint(&document);
        // Should handle code-switching gracefully
        assert!(
            lints.len() < 50,
            "Code-switching should not cause explosion of lints"
        );
    }

    /// Test performance: full workflow on realistic German paragraph
    #[test]
    fn test_full_workflow_performance() {
        let dict = FstDictionary::curated(); // English dictionary for detection

        // Realistic German paragraph with some errors
        let text = "Der Hund spielt im Garten mit dem Ball. \
                    die Katze schläft auf dem Sofa im Wohnzimmer. \
                    das Auto ist sehr schnell und fährt auf der Straße. \
                    Wir gehen heute ins Kino und essen danach im Restaurant.";

        let start = std::time::Instant::now();

        // Step 1: Detect
        let default_lang = Language::English(Dialect::American);
        let detected = detect_language(text, &dict, default_lang);

        // Step 2: Parse
        let parser = parser_for_prose("plaintext", detected, MarkdownOptions::default())
            .expect("Should get parser for detected language");
        let language_dict = dictionary(detected);
        let document = Document::new(text, &parser, &language_dict);

        // Step 3: Lint
        use harper_core::linting::Linter;
        let mut linter = new_curated_for_language(dict, detected);
        let lints = linter.lint(&document);

        let duration = start.elapsed();

        // Verify results
        assert_eq!(
            detected.family(),
            LanguageFamily::German,
            "Should detect German"
        );
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
}
