//! Language support framework for Harper.
//!
//! This module provides the core types for supporting multiple languages in Harper,
//! including language families and specific language variants with dialects.
use crate::language::english::dialects::EnglishDialect;
use crate::language::german::dialects::GermanDialect;
use crate::language::portuguese::dialects::PortugueseDialect;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumCount, EnumIter, EnumString};

/// Parse a language from a string representation.
/// This function handles various formats including:
/// - Abbreviations: "us", "uk", "de", "pt", "pt_br"
/// - Full names: "american", "british", "german", "portuguese", "brazilian"
/// - Locale codes: "en-US", "en-GB", "de-DE", "pt-BR"
pub fn parse_language(s: &str) -> Option<Language> {
    let s_lower = s.to_ascii_lowercase();

    // Try string matching
    match s_lower.as_str() {
        // English
        "us" | "usa" | "america" | "american" | "en-us" | "en_us" => {
            Some(Language::English(EnglishDialect::American))
        }
        "uk" | "gb" | "british" | "britain" | "en-gb" | "en_gb" => {
            Some(Language::English(EnglishDialect::British))
        }
        "au" | "aus" | "australia" | "australian" | "en-au" | "en_au" => {
            Some(Language::English(EnglishDialect::Australian))
        }
        "in" | "india" | "indian" | "bharat" | "en-in" | "en_in" => {
            Some(Language::English(EnglishDialect::Indian))
        }
        "ca" | "canada" | "canadian" | "en-ca" | "en_ca" => {
            Some(Language::English(EnglishDialect::Canadian))
        }
        // German
        "de" | "german" | "deutsch" | "de-de" | "de_de" => {
            Some(Language::German(GermanDialect::Standard))
        }
        "at" | "austria" | "austrian" | "de-at" | "de_at" => {
            Some(Language::German(GermanDialect::Austrian))
        }
        "ch" | "switzerland" | "swiss" | "de-ch" | "de_ch" => {
            Some(Language::German(GermanDialect::Swiss))
        }
        // Portuguese
        "pt" | "pt-pt" | "pt_pt" | "portuguese" | "portugu\u{00ea}s" => {
            Some(Language::Portuguese(PortugueseDialect::European))
        }
        "br" | "brazil" | "portuguese-brazilian" | "portuguese_brazilian" | "pt-br" | "pt_br" => {
            Some(Language::Portuguese(PortugueseDialect::Brazilian))
        }
        "ao" => Some(Language::Portuguese(PortugueseDialect::African)),
        _ => None,
    }
}

/// A specific language with its dialects.
///
/// This enum represents all supported languages in Harper, each with their specific dialect.
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, EnumCount, Display,
)]
pub enum Language {
    /// English language with its dialects
    English(EnglishDialect),
    /// German language with its dialects
    German(GermanDialect),
    /// Portuguese language with its dialects
    Portuguese(PortugueseDialect),
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

impl LanguageFamily {
    /// Returns a suffix to append to dictionary file paths for this language family.
    /// English returns `""` (default). German returns `"-de"`. Portuguese returns `"-pt"`.
    pub fn dict_suffix(&self) -> &'static str {
        match self {
            Self::German => "-de",
            Self::Portuguese => "-pt",
            Self::English => "",
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
        Self::English(EnglishDialect::American)
    }
}
