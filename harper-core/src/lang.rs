use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumDiscriminants, EnumIter, EnumString};

use crate::EnglishDialect;

/// A natural language Harper can check, together with its regional variant.
///
/// Code that only cares about the language, not the region, can use [`LanguageFamily`],
/// which has the same variants without the dialect: `language.family()`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(
    name(LanguageFamily),
    derive(Display, EnumString, EnumIter, Hash, Serialize, Deserialize),
    doc = "A natural language Harper can check, without its regional variant."
)]
pub enum Language {
    English(EnglishDialect),
}

impl Language {
    /// The language without its regional variant.
    pub fn family(self) -> LanguageFamily {
        self.into()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{Language, LanguageFamily};
    use crate::EnglishDialect;

    #[test]
    fn family_drops_the_dialect() {
        assert_eq!(
            Language::English(EnglishDialect::British).family(),
            LanguageFamily::English
        );
        assert_eq!(
            Language::English(EnglishDialect::American).family(),
            Language::English(EnglishDialect::Indian).family()
        );
    }

    #[test]
    fn family_round_trips_through_strings() {
        assert_eq!(
            LanguageFamily::from_str("English"),
            Ok(LanguageFamily::English)
        );
        assert_eq!(LanguageFamily::English.to_string(), "English");
    }

    #[test]
    fn language_round_trips_through_json() {
        let language = Language::English(EnglishDialect::Canadian);
        let json = serde_json::to_string(&language).unwrap();

        assert_eq!(json, r#"{"English":"Canadian"}"#);
        assert_eq!(serde_json::from_str::<Language>(&json).unwrap(), language);
    }
}
