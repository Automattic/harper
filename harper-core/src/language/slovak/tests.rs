//! Slovak language tests.
//!
//! This module contains tests for Slovak language functionality.
//! Tests are organized by component and can be run with `cargo test`.
//!
//! The module is already `#[cfg(test)]` where `mod.rs` declares it, so the
//! tests go here directly rather than inside a nested `mod tests`.

use crate::Document;
use crate::language::slovak::dialects::SlovakDialect;
use crate::language::slovak::linting::slovak_spell_check::SlovakSpellCheck;
use crate::language::slovak::parsers::PlainSlovak;
use crate::language::slovak::spell::curated_slovak_dictionary;
use crate::linting::Linter;

// Add Slovak-specific tests here when the language is fully integrated
// For now, this is a placeholder for future tests

#[test]
fn test_slovak_module_structure() {
    // This test verifies that the Slovak module can be imported
    // and basic functionality is available
    use crate::language::module::{LanguageDetector, LanguageModule};
    use crate::language::slovak::module::SlovakModule;

    // Test that we can get the default dialect
    let dialect = SlovakModule::default_dialect();
    assert_eq!(dialect, SlovakDialect::Standard);

    // Test that we can get the detector
    let detector = SlovakModule::detector();
    assert_eq!(detector.name(), "slovak");
}

fn spell_check_messages(text: &str) -> Vec<String> {
    let dictionary = curated_slovak_dictionary();
    let document = Document::new(text, &PlainSlovak, &dictionary);
    let mut check = SlovakSpellCheck::new(dictionary.clone(), SlovakDialect::default());
    check
        .lint(&document)
        .into_iter()
        .map(|lint| lint.message)
        .collect()
}

#[test]
fn misspelling_with_suggestions_has_slovak_message() {
    let messages = spell_check_messages("Dohodli sa nezavisle.");
    assert_eq!(messages.len(), 1, "got {messages:?}");
    assert!(
        messages[0].starts_with("Slovo „nezavisle“ môže byť napísané nesprávne. Mysleli ste: "),
        "got {messages:?}"
    );
    assert!(messages[0].contains("nezávisle"), "got {messages:?}");
}

#[test]
fn unknown_word_without_suggestions_has_slovak_message() {
    let messages = spell_check_messages("Dohodli sa qxzqxzqxzqxz.");
    assert_eq!(
        messages,
        vec!["Slovo „qxzqxzqxzqxz“ nie je v slovníku.".to_string()]
    );
}
