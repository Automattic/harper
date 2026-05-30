//! Language support framework for Harper.
//!
//! This module provides the core types for supporting multiple languages in Harper,
//! including language families and specific language variants with dialects.

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumCount, EnumIter, EnumString};

use crate::language::german::dialects::GermanDialect;
use crate::language::portuguese::dialects::PortugueseDialect;

/// A specific language with its dialect.
///
/// This enum represents all supported languages in Harper, each with their specific dialect.
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, EnumCount, Display,
)]
pub enum Language {
    /// English language with its dialects
    English(crate::Dialect),
    /// German language with its dialects
    German(GermanDialect),
    /// Portuguese language with its dialects
    Portuguese(PortugueseDialect),
}

impl Language {
    /// Creates a default Language (English with American dialect).
    pub fn default_english() -> Self {
        Self::English(crate::Dialect::American)
    }
}

/// A family of languages (e.g., English, German, Portuguese).
///
/// This is used when we need to identify the broad language category
/// without specifying a particular dialect.
#[derive(
    Default,
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    PartialOrd,
    Eq,
    Hash,
    EnumCount,
    EnumString,
    EnumIter,
    Display,
)]
pub enum LanguageFamily {
    /// English language family
    #[default]
    English,
    /// German language family
    German,
    /// Portuguese language family
    Portuguese,
}

impl From<Language> for LanguageFamily {
    fn from(value: Language) -> Self {
        match value {
            Language::English(_) => Self::English,
            Language::German(_) => Self::German,
            Language::Portuguese(_) => Self::Portuguese,
        }
    }
}

impl Language {
    /// Returns the language family for this language.
    pub fn family(&self) -> LanguageFamily {
        match self {
            Language::English(_) => LanguageFamily::English,
            Language::German(_) => LanguageFamily::German,
            Language::Portuguese(_) => LanguageFamily::Portuguese,
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        Self::default_english()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_family_conversion() {
        assert_eq!(
            LanguageFamily::from(Language::English(crate::Dialect::American)),
            LanguageFamily::English
        );
        assert_eq!(
            LanguageFamily::from(Language::German(GermanDialect::Standard)),
            LanguageFamily::German
        );
        // Portuguese tests will work once we create the PortugueseDialect enum
    }

    #[test]
    fn test_language_family_method() {
        assert_eq!(
            Language::English(crate::Dialect::British).family(),
            LanguageFamily::English
        );
        assert_eq!(
            Language::German(GermanDialect::Standard).family(),
            LanguageFamily::German
        );
    }

    #[test]
    fn test_default_language() {
        assert_eq!(Language::default().family(), LanguageFamily::English);
    }
}
