//! Language support framework for Harper.
//!
//! This module provides the core types for supporting multiple languages in Harper,
//! including language families and specific language variants with dialects.
use crate::language::english::dialects::EnglishDialect;

#[cfg(feature = "de")]
use crate::language::german::dialects::GermanDialect;

#[cfg(feature = "pt")]
use crate::language::portuguese::dialects::PortugueseDialect;

#[cfg(feature = "sk")]
use crate::language::slovak::dialects::SlovakDialect;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumCount, EnumIter, EnumString};

/// Parse a language from a string representation.
pub fn parse_language(s: &str) -> Option<Language> {
    let s_lower = s.to_ascii_lowercase();

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
        #[cfg(feature = "de")]
        "de" | "german" | "deutsch" | "de-de" | "de_de" => {
            Some(Language::German(GermanDialect::Standard))
        }
        #[cfg(feature = "de")]
        "at" | "austria" | "austrian" | "de-at" | "de_at" => {
            Some(Language::German(GermanDialect::Austrian))
        }
        #[cfg(feature = "de")]
        "ch" | "switzerland" | "swiss" | "de-ch" | "de_ch" => {
            Some(Language::German(GermanDialect::Swiss))
        }
        // Portuguese
        #[cfg(feature = "pt")]
        "pt" | "pt-pt" | "pt_pt" | "portuguese" | "portugu\u{00ea}s" => {
            Some(Language::Portuguese(PortugueseDialect::European))
        }
        #[cfg(feature = "pt")]
        "br" | "brazil" | "portuguese-brazilian" | "portuguese_brazilian" | "pt-br" | "pt_br" => {
            Some(Language::Portuguese(PortugueseDialect::Brazilian))
        }
        #[cfg(feature = "pt")]
        "ao" => Some(Language::Portuguese(PortugueseDialect::African)),
        // Slovak
        #[cfg(feature = "sk")]
        "sk" | "slovak" | "slovensko" | "sk-sk" | "sk_sk" => {
            Some(Language::Slovak(SlovakDialect::Standard))
        }
        _ => None,
    }
}

/// A specific language with its dialects.
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, EnumCount, Display,
)]
pub enum Language {
    /// English language with its dialects
    English(EnglishDialect),
    /// German language with its dialects
    #[cfg(feature = "de")]
    German(GermanDialect),
    /// Portuguese language with its dialects
    #[cfg(feature = "pt")]
    Portuguese(PortugueseDialect),
    /// Slovak language with its dialects
    #[cfg(feature = "sk")]
    Slovak(SlovakDialect),
}

/// A family of languages.
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
    #[default]
    English,
    #[cfg(feature = "de")]
    German,
    #[cfg(feature = "pt")]
    Portuguese,
    #[cfg(feature = "sk")]
    Slovak,
}

impl From<Language> for LanguageFamily {
    fn from(value: Language) -> Self {
        match value {
            Language::English(_) => Self::English,
            #[cfg(feature = "de")]
            Language::German(_) => Self::German,
            #[cfg(feature = "pt")]
            Language::Portuguese(_) => Self::Portuguese,
            #[cfg(feature = "sk")]
            Language::Slovak(_) => Self::Slovak,
        }
    }
}

impl LanguageFamily {
    pub fn dict_suffix(&self) -> &'static str {
        match self {
            Self::English => "",
            #[cfg(feature = "de")]
            Self::German => "-de",
            #[cfg(feature = "pt")]
            Self::Portuguese => "-pt",
            #[cfg(feature = "sk")]
            Self::Slovak => "-sk",
        }
    }
}

impl Language {
    pub fn family(&self) -> LanguageFamily {
        (*self).into()
    }

    /// Parse a language from a BCP 47 language tag.
    /// 
    /// BCP 47 tags are standardized language identifiers like "en-US", "de-DE", "pt-BR", etc.
    /// This method handles both the language code and region subtags.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use harper_core::language::Language;
    /// 
    /// let lang = Language::try_from_bcp47("en-US");
    /// assert_eq!(lang, Some(Language::English(EnglishDialect::American)));
    /// 
    /// let lang = Language::try_from_bcp47("de-DE");
    /// assert_eq!(lang, Some(Language::German(GermanDialect::Standard)));
    /// ```
    pub fn try_from_bcp47(bcp47: &str) -> Option<Self> {
        let bcp47_lower = bcp47.to_ascii_lowercase();
        let parts: Vec<&str> = bcp47_lower.split('-').collect();
        
        if parts.is_empty() {
            return None;
        }

        let lang_code = parts[0];
        let region = parts.get(1).copied();

        match (lang_code, region) {
            // English variants
            ("en", Some("us") | Some("usa") | Some("america") | Some("american")) => {
                Some(Self::English(EnglishDialect::American))
            }
            ("en", Some("gb") | Some("uk") | Some("british") | Some("britain")) => {
                Some(Self::English(EnglishDialect::British))
            }
            ("en", Some("au") | Some("aus") | Some("australia") | Some("australian")) => {
                Some(Self::English(EnglishDialect::Australian))
            }
            ("en", Some("ca") | Some("canada") | Some("canadian")) => {
                Some(Self::English(EnglishDialect::Canadian))
            }
            ("en", Some("in") | Some("india") | Some("indian") | Some("bharat")) => {
                Some(Self::English(EnglishDialect::Indian))
            }
            ("en", _) => Some(Self::English(EnglishDialect::American)), // Default English
            
            // German variants
            #[cfg(feature = "de")]
            ("de", Some("de") | Some("germany") | Some("deutsch") | None) => {
                Some(Self::German(GermanDialect::Standard))
            }
            #[cfg(feature = "de")]
            ("de", Some("at") | Some("austria") | Some("austrian")) => {
                Some(Self::German(GermanDialect::Austrian))
            }
            #[cfg(feature = "de")]
            ("de", Some("ch") | Some("switzerland") | Some("swiss")) => {
                Some(Self::German(GermanDialect::Swiss))
            }
            
            // Portuguese variants
            #[cfg(feature = "pt")]
            ("pt", Some("pt") | Some("portugal") | None) => {
                Some(Self::Portuguese(PortugueseDialect::European))
            }
            #[cfg(feature = "pt")]
            ("pt", Some("br") | Some("brazil") | Some("brazilian")) => {
                Some(Self::Portuguese(PortugueseDialect::Brazilian))
            }
            #[cfg(feature = "pt")]
            ("pt", Some("ao") | Some("angola") | Some("african")) => {
                Some(Self::Portuguese(PortugueseDialect::African))
            }
            
            // Slovak variants
            #[cfg(feature = "sk")]
            ("sk", _) => Some(Self::Slovak(SlovakDialect::Standard)),
            
            // If we only have a language code without region, try to match it
            (lang, None) => parse_language(lang),
            
            _ => None,
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        Self::English(EnglishDialect::American)
    }
}
