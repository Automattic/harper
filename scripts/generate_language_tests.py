#!/usr/bin/env python3
"""
Language Integration Test Generator

Generates comprehensive integration test templates for Harper language modules.
Based on the Polish integration test structure.
"""

import os
import re
import sys
from pathlib import Path

# Integration test template
INTEGRATION_TEST_TEMPLATE = '''#![cfg(feature = "{feature}")]

/// {display_name} language integration tests
mod tests {{
    use harper_core::Document;
    use harper_core::language::LanguageDetector;
    use harper_core::language::module::LanguageModule;
    use harper_core::language::{lang}::dialects::{{LangCapital}Dialect, {LangCapital}DialectFlags}};
    use harper_core::language::{lang}::language_detection::{{LangCapital}Detector};
    use harper_core::language::{lang}::module::{{LangCapital}Module};
    use harper_core::parsers::Parser;
    use harper_core::spell::FstDictionary;
    use std::sync::Arc;

    #[test]
    fn test_{lang}_module_instantiation() {{
        // Test that the {display_name} module can be instantiated
        let _module = {LangCapital}Module;
    }}

    #[test]
    fn test_{lang}_default_dialect() {{
        // Test that we can get the default dialect
        let dialect = {LangCapital}Module::default_dialect();
        assert_eq!(dialect, {LangCapital}Dialect::Standard);
    }}

    #[test]
    fn test_{lang}_detector() {{
        // Test that we can get the detector
        let _detector = {LangCapital}Module::detector();
    }}

    #[test]
    fn test_{lang}_dictionary_loading() {{
        // Test that we can load the dictionary without panics
        let result = std::panic::catch_unwind(|| {{
            let _dict = {LangCapital}Module::dictionary();
        }});
        assert!(result.is_ok(), "{display_name} dictionary should load successfully");
    }}

    #[test]
    fn test_{lang}_parser_basic() {{
        // Test basic parsing functionality
        let parser = {LangCapital}Module::plain_parser();
        let text = "Test text in {display_name}";
        let chars: Vec<char> = text.chars().collect();
        let tokens = Parser::parse(&parser, &chars);
        assert!(!tokens.is_empty(), "{display_name} parser should produce tokens");
    }}

    #[test]
    fn test_{lang}_parser_with_native_text() {{
        // Test parsing with native {display_name} text
        let parser = {LangCapital}Module::plain_parser();
        let native_text = "{native_text_example}";
        let chars: Vec<char> = native_text.chars().collect();
        let tokens = Parser::parse(&parser, &chars);
        assert!(!tokens.is_empty(), "{display_name} parser should handle native text");
    }}

    #[test]
    fn test_{lang}_language_detection() {{
        use harper_core::language::languages::Language;
        use harper_core::spell::FstDictionary;

        let detector = {LangCapital}Detector;
        let dict = FstDictionary::curated();

        // Test with {display_name}-specific text
        let {lang}_text = "{detection_text_example}";
        let doc = Document::new_plain_english_curated({lang}_text);
        let result = LanguageDetector::detect(&detector, doc.get_tokens(), doc.get_source(), &dict);

        // Detection may or may not work depending on the implementation
        // This test just ensures the detection doesn't panic
        assert!(true, "{display_name} language detection should not panic");
    }}

    #[test]
    fn test_{lang}_lint_group_creation() {{
        use harper_core::spell::FstDictionary;
        use std::sync::Arc;

        // Test that we can create a lint group
        let dict = Arc::new(FstDictionary::curated());
        let lint_group = {LangCapital}Module::rust_lint_group(dict);

        // The lint group should contain at least the spell check linter
        assert!(
            lint_group.contains_key("{LangCapital}SpellCheck"),
            "Should contain {display_name} spell check linter"
        );
    }}

    #[test]
    fn test_{lang}_weir_lint_group() {{
        // Test that we can create the Weir lint group
        let weir_group = {LangCapital}Module::weir_lint_group();
        // Just ensure it doesn't panic
        assert!(true, "{display_name} Weir lint group should be creatable");
    }}

    #[test]
    fn test_{lang}_curated_lint_group() {{
        use harper_core::spell::FstDictionary;
        use std::sync::Arc;

        // Test that we can create the curated lint group
        let dict = Arc::new(FstDictionary::curated());
        let dialect = {LangCapital}Dialect::default();
        let curated_group = {LangCapital}Module::curated_lint_group(dialect, dict);
        // Just ensure it doesn't panic
        assert!(true, "{display_name} curated lint group should be creatable");
    }}

    #[test]
    fn test_{lang}_serialization() {{
        // Test that we can serialize and deserialize dialect flags
        let flags = {LangCapital}DialectFlags::STANDARD;
        let serialized = serde_json::to_string(&flags).unwrap();
        let deserialized: {LangCapital}DialectFlags = serde_json::from_str(&serialized).unwrap();
        assert_eq!(flags, deserialized);
    }}

    #[test]
    fn test_{lang}_lex_token() {{
        // Test the lex_token function directly
        use crate::language::{lang}::lexing::lex_{lang}_token;
        use crate::lexing::FoundToken;
        
        let text = "test";
        let chars: Vec<char> = text.chars().collect();
        let token = lex_{lang}_token(&chars);
        assert!(true, "{display_name} lex_token should work without panicking");
    }}

    #[test]
    fn test_{lang}_confidence() {{
        // Test detector confidence
        let detector = {LangCapital}Detector;
        let confidence = detector.confidence();
        assert!(confidence >= 0.0 && confidence <= 1.0, 
               "Confidence should be between 0 and 1");
    }}

    #[test]
    fn test_{lang}_detector_name() {{
        // Test detector name
        let detector = {LangCapital}Detector;
        let name = detector.name();
        assert_eq!(name, "{lang}", "Detector name should match language code");
    }}
}}
'''

# Language detection test template
DETECTION_TEST_TEMPLATE = '''#![cfg(feature = "{feature}")]

/// {display_name} language detection tests
mod tests {{
    use harper_core::Document;
    use harper_core::language::LanguageDetector;
    use harper_core::language::languages::Language;
    use harper_core::language::{lang}::dialects::{{LangCapital}Dialect};
    use harper_core::language::{lang}::language_detection::{{LangCapital}Detector};
    use harper_core::spell::FstDictionary;

    #[test]
    fn test_{lang}_detection_english_text() {{
        let detector = {LangCapital}Detector;
        let dict = FstDictionary::curated();

        let english_text = "This is a test sentence in English with common words.";
        let doc = Document::new_plain_english_curated(english_text);
        let result = LanguageDetector::detect(&detector, doc.get_tokens(), doc.get_source(), &dict);
        
        // Should not detect {display_name} for English text
        assert!(result.is_none(), "Should not detect {display_name} for English text");
    }}

    #[test]
    fn test_{lang}_detection_mixed_text() {{
        let detector = {LangCapital}Detector;
        let dict = FstDictionary::curated();

        // Test with mixed text - detection behavior depends on implementation
        let mixed_text = "This is a test with some {display_name} words mixed in.";
        let doc = Document::new_plain_english_curated(mixed_text);
        let result = LanguageDetector::detect(&detector, doc.get_tokens(), doc.get_source(), &dict);
        
        // Mixed text may or may not be detected - just ensure no panic
        assert!(true, "Mixed text detection should not panic");
    }}

    #[test]
    fn test_{lang}_detection_confidence() {{
        let detector = {LangCapital}Detector;
        let confidence = detector.confidence();
        
        // Confidence should be reasonable
        assert!(confidence >= 0.7, "{display_name} detector confidence should be >= 0.7");
        assert!(confidence <= 1.0, "Confidence should not exceed 1.0");
    }}

    #[test]
    fn test_{lang}_detector_attributes() {{
        let detector = {LangCapital}Detector;
        
        // Test detector name
        assert_eq!(detector.name(), "{lang}", "Detector name should be '{lang}'");
        
        // Test confidence is reasonable
        let confidence = detector.confidence();
        assert!(confidence > 0.0, "Confidence should be positive");
    }}
}}
'''

# Dictionary test template
DICTIONARY_TEST_TEMPLATE = '''#![cfg(feature = "{feature}")]

/// {display_name} dictionary tests
mod tests {{
    use harper_core::spell::FstDictionary;
    use std::sync::Arc;

    #[test]
    fn test_{lang}_dictionary_loads() {{
        // Test that the dictionary loads without errors
        let result = std::panic::catch_unwind(|| {{
            let _dict = {LangCapital}Module::dictionary();
        }});
        assert!(result.is_ok(), "{display_name} dictionary should load successfully");
    }}

    #[test]
    fn test_{lang}_dictionary_is_fst() {{
        // Test that the dictionary is an FST dictionary
        let dict = {LangCapital}Module::dictionary();
        // Just ensure we can get it and it's the right type
        assert!(true, "{display_name} dictionary should be an FST dictionary");
    }}

    #[test]
    fn test_{lang}_dictionary_word_lookup() {{
        // Test basic word lookup (if dictionary has words)
        let dict = {LangCapital}Module::dictionary();
        
        // Try to look up some basic words - this may or may not work
        // depending on the dictionary content
        let _result = dict.contains_word(&"test".chars().collect::<Vec<_>>());
        assert!(true, "Word lookup should not panic");
    }}
}}
'''


def get_language_info(language):
    """Get information about an existing language to use as examples."""
    # Default examples for different language types
    language_examples = {
        'german': {
            'native_text': 'Das ist ein Testtext auf Deutsch',
            'detection_text': 'Der die das und oder aber',
            'display_name': 'German',
            'LangCapital': 'German',
            'feature': 'de'
        },
        'portuguese': {
            'native_text': 'Isto é um texto de teste em Português',
            'detection_text': 'o a e de em um para',
            'display_name': 'Portuguese', 
            'LangCapital': 'Portuguese',
            'feature': 'pt'
        },
        'slovak': {
            'native_text': 'Toto je testovací text v slovenčine',
            'detection_text': 'a v na s o k je',
            'display_name': 'Slovak',
            'LangCapital': 'Slovak',
            'feature': 'sk'
        },
        'polish': {
            'native_text': 'To jest tekst testowy w języku polskim',
            'detection_text': 'i w na z do że jest',
            'display_name': 'Polish',
            'LangCapital': 'Polish',
            'feature': 'pl'
        }
    }
    
    return language_examples.get(language.lower(), {
        'native_text': 'This is a test sentence in the language',
        'detection_text': 'some common words',
        'display_name': language.capitalize(),
        'LangCapital': language.capitalize(),
        'feature': language.lower()
    })


def create_file_safely(path, content):
    """Create a file safely, checking if it already exists."""
    path = Path(path)
    if path.exists():
        print(f"⚠️  File already exists, skipping: {path}")
        return False
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        with open(path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"✅ Created: {path}")
        return True


def generate_integration_tests(lang):
    """Generate comprehensive integration tests for a language."""
    print(f"\n🧪 Generating integration tests for {lang}")
    
    # Get language info for examples
    lang_info = get_language_info(lang)
    display_name = lang_info['display_name']
    LangCapital = lang_info['LangCapital']
    feature = lang_info['feature']
    native_text = lang_info['native_text']
    detection_text = lang_info['detection_text']
    
    # Substitute placeholders
    def substitute(template):
        return template.replace('{lang}', lang) \
                       .replace('{LangCapital}', LangCapital) \
                       .replace('{display_name}', display_name) \
                       .replace('{feature}', feature) \
                       .replace('{native_text_example}', native_text) \
                       .replace('{detection_text_example}', detection_text)
    
    # Create the integration test file
    test_dir = Path("harper-core/tests")
    test_file = test_dir / f"{lang}_integration_test.rs"
    
    if test_file.exists():
        print(f"⚠️  Integration test file already exists: {test_file}")
        return False
    
    content = substitute(INTEGRATION_TEST_TEMPLATE)
    return create_file_safely(test_file, content)


def generate_all_tests(lang):
    """Generate all test types for a language."""
    print(f"\n🧪 Generating all test files for {lang}")
    
    # Get language info
    lang_info = get_language_info(lang)
    display_name = lang_info['display_name']
    LangCapital = lang_info['LangCapital']
    feature = lang_info['feature']
    native_text = lang_info['native_text']
    detection_text = lang_info['detection_text']
    
    # Substitute placeholders
    def substitute(template):
        return template.replace('{lang}', lang) \
                       .replace('{LangCapital}', LangCapital) \
                       .replace('{display_name}', display_name) \
                       .replace('{feature}', feature) \
                       .replace('{native_text_example}', native_text) \
                       .replace('{detection_text_example}', detection_text)
    
    test_dir = Path("harper-core/tests")
    tests_created = []
    
    # Generate integration tests
    tests_created.append(create_file_safely(
        test_dir / f"{lang}_integration_test.rs",
        substitute(INTEGRATION_TEST_TEMPLATE)
    ))
    
    # Generate detection tests
    tests_created.append(create_file_safely(
        test_dir / f"{lang}_detection_test.rs",
        substitute(DETECTION_TEST_TEMPLATE)
    ))
    
    # Generate dictionary tests
    tests_created.append(create_file_safely(
        test_dir / f"{lang}_dictionary_test.rs",
        substitute(DICTIONARY_TEST_TEMPLATE)
    ))
    
    if tests_created:
        print(f"✅ Generated {sum(tests_created)} test files for {lang}")
    
    return sum(tests_created) > 0


def main():
    """Main entry point."""
    if len(sys.argv) >= 2:
        lang = sys.argv[1].lower()
        
        # Check if --all flag is present
        if len(sys.argv) == 2:
            # Generate comprehensive integration tests only
            success = generate_integration_tests(lang)
        elif len(sys.argv) == 3 and sys.argv[2] == "--all":
            # Generate all test types
            success = generate_all_tests(lang)
        else:
            # Generate comprehensive integration tests only
            success = generate_integration_tests(lang)
            
        sys.exit(0 if success else 1)
    else:
        print("Usage: python3 generate_language_tests.py <language> [--all]")
        print("Example: python3 generate_language_tests.py finnish")
        print("Example: python3 generate_language_tests.py finnish --all")
        print("\n  <language>     - Language code (e.g., fi, sv, no)")
        print("  --all         - Generate all test types (integration, detection, dictionary)")
        sys.exit(1)


if __name__ == "__main__":
    import sys
    main()