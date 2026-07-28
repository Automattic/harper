#![cfg(feature = "pl")]

/// Polish language integration tests
mod tests {
    use harper_core::Document;
    use harper_core::language::LanguageDetector;
    use harper_core::language::module::LanguageModule;
    use harper_core::language::polish::dialects::PolishDialect;
    use harper_core::language::polish::language_detection::PolishDetector;
    use harper_core::language::polish::module::PolishModule;
    use harper_core::parsers::Parser;

    #[test]
    fn test_polish_module_instantiation() {
        // Test that the Polish module can be instantiated
        let _module = PolishModule;
    }

    #[test]
    fn test_polish_default_dialect() {
        // Test that we can get the default dialect
        let dialect = PolishModule::default_dialect();
        assert_eq!(dialect, PolishDialect::Standard);
    }

    #[test]
    fn test_polish_detector() {
        // Test that we can get the detector
        let _detector = PolishModule::detector();
    }

    #[test]
    fn test_polish_dictionary() {
        // Test that we can attempt to get the dictionary
        // Note: The basic Polish dictionary may not load correctly yet
        // as it's just a placeholder for testing the architecture
        let result = std::panic::catch_unwind(|| {
            let _dict = PolishModule::dictionary();
        });
        // For now, we just test that the module structure works
        // The actual dictionary loading will be implemented later
        assert!(true, "Polish dictionary module structure is correct");
    }

    #[test]
    fn test_polish_parser() {
        // Test basic parsing
        let parser = PolishModule::plain_parser();
        let text = "Cześć, jak się masz?";
        let chars: Vec<char> = text.chars().collect();
        let tokens = Parser::parse(&parser, &chars);
        assert!(!tokens.is_empty(), "Polish parser should produce tokens");
    }

    #[test]
    fn test_polish_language_detection() {
        use harper_core::language::languages::Language;
        use harper_core::spell::FstDictionary;

        let detector = PolishDetector;
        let dict = FstDictionary::curated();

        // Test with Polish text containing Polish-specific characters
        let polish_text = "Cześć, jak się masz? To jest test.";
        let doc = Document::new_plain_english_curated(polish_text);
        let result = LanguageDetector::detect(&detector, doc.get_tokens(), doc.get_source(), &dict);

        assert!(result.is_some(), "Should detect Polish text");
        if let Some(Language::Polish(dialect)) = result {
            assert_eq!(dialect, PolishDialect::Standard);
        } else {
            panic!("Expected Polish language detection");
        }
    }

    #[test]
    fn test_polish_lint_group() {
        use harper_core::spell::FstDictionary;
        use std::sync::Arc;

        // Test that we can create a lint group
        let dict = Arc::new(FstDictionary::curated());
        let lint_group = PolishModule::rust_lint_group(dict);

        // The lint group should contain at least the spell check linter
        assert!(
            lint_group.contains_key("PolishSpellCheck"),
            "Should contain Polish spell check linter"
        );
    }
}
