//! Slovak language tests.
//!
//! This module contains tests for Slovak language functionality.
//! Tests are organized by component and can be run with `cargo test`.

#[cfg(test)]
mod tests {
    // Add Slovak-specific tests here when the language is fully integrated
    // For now, this is a placeholder for future tests

    #[test]
    fn test_slovak_module_structure() {
        // This test verifies that the Slovak module can be imported
        // and basic functionality is available
        use crate::language::module::{LanguageDetector, LanguageModule};
        use crate::language::slovak::dialects::SlovakDialect;
        use crate::language::slovak::module::SlovakModule;

        // Test that we can get the default dialect
        let dialect = SlovakModule::default_dialect();
        assert_eq!(dialect, SlovakDialect::Standard);

        // Test that we can get the detector
        let detector = SlovakModule::detector();
        assert_eq!(detector.name(), "slovak");
    }
}
