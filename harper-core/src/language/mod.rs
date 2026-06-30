//! Language-specific modules for Harper.
//!
//! This module organizes language-specific functionality using the LanguageModule trait.

pub mod dialects;
pub mod languages;
pub mod module;
pub mod registry;

pub mod english;
pub mod german;
pub mod portuguese;
pub mod slovak;

// Re-export core types
pub use languages::{Language, LanguageFamily, parse_language};
pub use module::{LanguageDetector, LanguageModule};

// Re-export registry functions
pub use registry::{
    ProseLanguage, add_language_specific_linters, detect_language, dictionary,
    dictionary_for_language, new_curated_for_language, parser_for_prose, prose_language,
    weir_rules_lint_group,
};
