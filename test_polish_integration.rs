// Simple test to verify Polish language integration
#[cfg(feature = "pl")]
#[test]
fn test_polish_language_integration() {
    use harper_core::language::polish::dialects::PolishDialect;
    use harper_core::language::polish::language_detection::PolishDetector;
    use harper_core::language::polish::module::PolishModule;
    use harper_core::language::module::LanguageModule;
    use harper_core::Document;
    
    // Test that the Polish module can be instantiated
    let _module = PolishModule;
    
    // Test that we can get the default dialect
    let _dialect = PolishModule::default_dialect();
    
    // Test that we can get the detector
    let _detector = PolishModule::detector();
    
    // Test that we can get the dictionary
    let _dict = PolishModule::dictionary();
    
    // Test basic parsing
    let parser = PolishModule::plain_parser();
    let text = "Cześć, jak się masz?";
    let chars: Vec<char> = text.chars().collect();
    let tokens = parser.parse(&chars);
    assert!(!tokens.is_empty(), "Polish parser should produce tokens");
    
    println!("Polish language integration test passed!");
}