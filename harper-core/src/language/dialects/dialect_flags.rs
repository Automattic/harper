use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Import dialect types from the central dialects module for modularity.
use crate::language::dialects::dialect_trait::DialectFlags as _;
use crate::language::english::dialects::{EnglishDialect, EnglishDialectFlags};
use crate::language::german::dialects::{GermanDialect, GermanDialectFlags};
use crate::language::portuguese::dialects::{PortugueseDialect, PortugueseDialectFlags};
use crate::language::slovak::dialects::{SlovakDialect, SlovakDialectFlags};

/// This represents a collection of dialect flags for all supported languages.
/// Each language has its own set of dialect flags.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
pub struct DialectFlags {
    // IMPORTANT: These fields must match the LANGUAGES! macro in dict_word_metadata.rs.
    // To add a new language, add a field here and update the LANGUAGES! macro.
    pub english: EnglishDialectFlags,
    pub german: GermanDialectFlags,
    pub portuguese: PortugueseDialectFlags,
    pub slovak: SlovakDialectFlags,
}

impl Serialize for DialectFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut scoped = serializer.serialize_struct("DialectFlags", 4)?;
        scoped.serialize_field("english", &self.english)?;
        scoped.serialize_field("german", &self.german)?;
        scoped.serialize_field("portuguese", &self.portuguese)?;
        scoped.serialize_field("slovak", &self.slovak)?;
        scoped.end()
    }
}

impl<'de> Deserialize<'de> for DialectFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Only accept the new scoped, language-specific dialect flags format.
        let scoped = ScopedDialectFlagsSerde::deserialize(deserializer)?;
        Ok(scoped.into())
    }
}

impl From<ScopedDialectFlagsSerde> for DialectFlags {
    fn from(value: ScopedDialectFlagsSerde) -> Self {
        Self {
            english: value.english,
            german: value.german,
            portuguese: value.portuguese,
            slovak: value.slovak,
        }
    }
}

impl DialectFlags {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            english: EnglishDialectFlags::empty(),
            german: GermanDialectFlags::empty(),
            portuguese: PortugueseDialectFlags::empty(),
            slovak: SlovakDialectFlags::empty(),
        }
    }

    /// Creates a DialectFlags with the specified English, German, Portuguese, and Slovak dialect flags.
    #[must_use]
    pub const fn new(
        english: EnglishDialectFlags,
        german: GermanDialectFlags,
        portuguese: PortugueseDialectFlags,
        slovak: SlovakDialectFlags,
    ) -> Self {
        Self {
            english,
            german,
            portuguese,
            slovak,
        }
    }

    /// Creates a DialectFlags with only the specified English dialect enabled.
    /// This is a convenience method for tests and cases where only English dialects are needed.
    #[must_use]
    pub fn from_english_dialect(dialect: EnglishDialect) -> Self {
        let english_flags = match dialect {
            EnglishDialect::American => EnglishDialectFlags::AMERICAN,
            EnglishDialect::Canadian => EnglishDialectFlags::CANADIAN,
            EnglishDialect::Australian => EnglishDialectFlags::AUSTRALIAN,
            EnglishDialect::British => EnglishDialectFlags::BRITISH,
            EnglishDialect::Indian => EnglishDialectFlags::INDIAN,
        };

        Self {
            english: english_flags,
            german: GermanDialectFlags::empty(),
            portuguese: PortugueseDialectFlags::empty(),
            slovak: SlovakDialectFlags::empty(),
        }
    }

    #[must_use]
    pub fn is_empty(self) -> bool {
        self.english.is_empty() && self.german.is_empty() && self.portuguese.is_empty() && self.slovak.is_empty()
    }

    #[must_use]
    pub fn is_english_dialect_enabled(self, dialect: EnglishDialect) -> bool {
        self.english.is_dialect_enabled(dialect)
    }

    #[must_use]
    pub fn is_english_dialect_enabled_strict(self, dialect: EnglishDialect) -> bool {
        self.english.is_dialect_enabled_strict(dialect)
    }

    #[must_use]
    pub fn is_german_dialect_enabled(self, dialect: GermanDialect) -> bool {
        self.german.is_dialect_enabled(dialect)
    }

    #[must_use]
    pub fn is_german_dialect_enabled_strict(self, dialect: GermanDialect) -> bool {
        self.german.is_dialect_enabled_strict(dialect)
    }

    #[must_use]
    pub fn is_portuguese_dialect_enabled(self, dialect: PortugueseDialect) -> bool {
        self.portuguese.is_dialect_enabled(dialect)
    }

    #[must_use]
    pub fn is_portuguese_dialect_enabled_strict(self, dialect: PortugueseDialect) -> bool {
        self.portuguese.is_dialect_enabled_strict(dialect)
    }

    #[must_use]
    pub fn is_slovak_dialect_enabled(self, dialect: SlovakDialect) -> bool {
        self.slovak.is_dialect_enabled(dialect)
    }

    #[must_use]
    pub fn is_slovak_dialect_enabled_strict(self, dialect: SlovakDialect) -> bool {
        self.slovak.is_dialect_enabled_strict(dialect)
    }

    /// Gets the most commonly used dialect(s) in the document.
    ///
    /// If multiple dialects are used equally often, they will all be enabled in the returned
    /// `DialectFlags`. On the other hand, if there is a single dialect that is used the most, it
    /// will be the only one enabled.
    #[must_use]
    pub fn get_most_used_dialects_from_document(document: &crate::Document) -> Self {
        // Get the most used dialects for each language separately
        let english_flags = EnglishDialectFlags::get_most_used_dialects_from_document(document);
        let german_flags = GermanDialectFlags::get_most_used_dialects_from_document(document);
        let portuguese_flags =
            PortugueseDialectFlags::get_most_used_dialects_from_document(document);
        let slovak_flags =
            SlovakDialectFlags::get_most_used_dialects_from_document(document);

        Self {
            english: english_flags,
            german: german_flags,
            portuguese: portuguese_flags,
            slovak: slovak_flags,
        }
    }
}

impl std::ops::BitOr for DialectFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            english: self.english | rhs.english,
            german: self.german | rhs.german,
            portuguese: self.portuguese | rhs.portuguese,
            slovak: self.slovak | rhs.slovak,
        }
    }
}

impl std::ops::BitOrAssign for DialectFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.english |= rhs.english;
        self.german |= rhs.german;
        self.portuguese |= rhs.portuguese;
        self.slovak |= rhs.slovak;
    }
}

impl Default for DialectFlags {
    /// A default value with no dialects explicitly enabled.
    /// Implicitly, this state corresponds to all dialects being enabled.
    fn default() -> Self {
        Self::empty()
    }
}

// Old legacy support (numeric bitmasks and flat strings) has been removed to simplify the data model.
// Use the ScopedDialectFlagsSerde and DialectFlags (language-scoped) for serialization/deserialization.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash, Default)]
struct ScopedDialectFlagsSerde {
    english: EnglishDialectFlags,
    german: GermanDialectFlags,
    portuguese: PortugueseDialectFlags,
    slovak: SlovakDialectFlags,
}

impl<'de> Deserialize<'de> for ScopedDialectFlagsSerde {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, Unexpected};

        let value = Value::deserialize(deserializer)?;

        match value {
            Value::Object(map) => {
                let mut english = EnglishDialectFlags::default();
                let mut german = GermanDialectFlags::default();
                let mut portuguese = PortugueseDialectFlags::default();
                let mut slovak = SlovakDialectFlags::default();

                for (key, val) in map {
                    match key.as_str() {
                        "english" => match val {
                            Value::String(s) => {
                                english = match s.as_str() {
                                    "AMERICAN" => EnglishDialectFlags::AMERICAN,
                                    "CANADIAN" => EnglishDialectFlags::CANADIAN,
                                    "AUSTRALIAN" => EnglishDialectFlags::AUSTRALIAN,
                                    "BRITISH" => EnglishDialectFlags::BRITISH,
                                    "INDIAN" => EnglishDialectFlags::INDIAN,
                                    _ => {
                                        return Err(Error::custom(format!(
                                            "Unknown English dialect: {s}"
                                        )));
                                    }
                                };
                            }
                            Value::Number(n) => {
                                let num =
                                    n.as_u64().ok_or_else(|| Error::custom("Invalid number"))?
                                        as u8;
                                english = EnglishDialectFlags::from_bits(num)
                                    .ok_or_else(|| Error::custom("Invalid dialect flags"))?;
                            }
                            _ => {
                                return Err(Error::invalid_type(
                                    Unexpected::Other("english"),
                                    &"string or number",
                                ));
                            }
                        },
                        "german" => match val {
                            Value::String(s) => {
                                german = match s.as_str() {
                                    "STANDARD" => GermanDialectFlags::STANDARD,
                                    "AUSTRIAN" => GermanDialectFlags::AUSTRIAN,
                                    "SWISS" => GermanDialectFlags::SWISS,
                                    _ => {
                                        return Err(Error::custom(format!(
                                            "Unknown German dialect: {s}"
                                        )));
                                    }
                                };
                            }
                            Value::Number(n) => {
                                let num =
                                    n.as_u64().ok_or_else(|| Error::custom("Invalid number"))?
                                        as u8;
                                german = GermanDialectFlags::from_bits(num)
                                    .ok_or_else(|| Error::custom("Invalid dialect flags"))?;
                            }
                            _ => {
                                return Err(Error::invalid_type(
                                    Unexpected::Other("german"),
                                    &"string or number",
                                ));
                            }
                        },
                        "portuguese" => match val {
                            Value::String(s) => {
                                portuguese = match s.as_str() {
                                    "EUROPEAN" => PortugueseDialectFlags::EUROPEAN,
                                    "BRAZILIAN" => PortugueseDialectFlags::BRAZILIAN,
                                    "AFRICAN" => PortugueseDialectFlags::AFRICAN,
                                    _ => {
                                        return Err(Error::custom(format!(
                                            "Unknown Portuguese dialect: {s}"
                                        )));
                                    }
                                };
                            }
                            Value::Number(n) => {
                                let num =
                                    n.as_u64().ok_or_else(|| Error::custom("Invalid number"))?
                                        as u8;
                                portuguese = PortugueseDialectFlags::from_bits(num)
                                    .ok_or_else(|| Error::custom("Invalid dialect flags"))?;
                            }
                            _ => {
                                return Err(Error::invalid_type(
                                    Unexpected::Other("portuguese"),
                                    &"string or number",
                                ));
                            }
                        },
                        "slovak" => match val {
                            Value::String(s) => {
                                slovak = match s.as_str() {
                                    "STANDARD" => SlovakDialectFlags::STANDARD,
                                    _ => {
                                        return Err(Error::custom(format!(
                                            "Unknown Slovak dialect: {s}"
                                        )));
                                    }
                                };
                            }
                            Value::Number(n) => {
                                let num =
                                    n.as_u64().ok_or_else(|| Error::custom("Invalid number"))?
                                        as u8;
                                slovak = SlovakDialectFlags::from_bits(num)
                                    .ok_or_else(|| Error::custom("Invalid dialect flags"))?;
                            }
                            _ => {
                                return Err(Error::invalid_type(
                                    Unexpected::Other("slovak"),
                                    &"string or number",
                                ));
                            }
                        },
                        _ => {
                            return Err(Error::unknown_field(
                                &key,
                                &["english", "german", "portuguese", "slovak"],
                            ));
                        }
                    }
                }

                Ok(ScopedDialectFlagsSerde {
                    english,
                    german,
                    portuguese,
                    slovak,
                })
            }
            Value::String(s) => {
                // Legacy format: single string representing one dialect
                match s.as_str() {
                    "AMERICAN" => Ok(ScopedDialectFlagsSerde {
                        english: EnglishDialectFlags::AMERICAN,
                        german: GermanDialectFlags::default(),
                        portuguese: PortugueseDialectFlags::default(),
                        slovak: SlovakDialectFlags::default(),
                    }),
                    "CANADIAN" => Ok(ScopedDialectFlagsSerde {
                        english: EnglishDialectFlags::CANADIAN,
                        german: GermanDialectFlags::default(),
                        portuguese: PortugueseDialectFlags::default(),
                        slovak: SlovakDialectFlags::default(),
                    }),
                    "AUSTRALIAN" => Ok(ScopedDialectFlagsSerde {
                        english: EnglishDialectFlags::AUSTRALIAN,
                        german: GermanDialectFlags::default(),
                        portuguese: PortugueseDialectFlags::default(),
                        slovak: SlovakDialectFlags::default(),
                    }),
                    "BRITISH" => Ok(ScopedDialectFlagsSerde {
                        english: EnglishDialectFlags::BRITISH,
                        german: GermanDialectFlags::default(),
                        portuguese: PortugueseDialectFlags::default(),
                        slovak: SlovakDialectFlags::default(),
                    }),
                    "INDIAN" => Ok(ScopedDialectFlagsSerde {
                        english: EnglishDialectFlags::INDIAN,
                        german: GermanDialectFlags::default(),
                        portuguese: PortugueseDialectFlags::default(),
                        slovak: SlovakDialectFlags::default(),
                    }),
                    "STANDARD" => Ok(ScopedDialectFlagsSerde {
                        english: EnglishDialectFlags::default(),
                        german: GermanDialectFlags::STANDARD,
                        portuguese: PortugueseDialectFlags::default(),
                        slovak: SlovakDialectFlags::default(),
                    }),
                    "AUSTRIAN" => Ok(ScopedDialectFlagsSerde {
                        english: EnglishDialectFlags::default(),
                        german: GermanDialectFlags::AUSTRIAN,
                        portuguese: PortugueseDialectFlags::default(),
                        slovak: SlovakDialectFlags::default(),
                    }),
                    "SWISS" => Ok(ScopedDialectFlagsSerde {
                        english: EnglishDialectFlags::default(),
                        german: GermanDialectFlags::SWISS,
                        portuguese: PortugueseDialectFlags::default(),
                        slovak: SlovakDialectFlags::default(),
                    }),
                    "EUROPEAN" => Ok(ScopedDialectFlagsSerde {
                        english: EnglishDialectFlags::default(),
                        german: GermanDialectFlags::default(),
                        portuguese: PortugueseDialectFlags::EUROPEAN,
                        slovak: SlovakDialectFlags::default(),
                    }),
                    "BRAZILIAN" => Ok(ScopedDialectFlagsSerde {
                        english: EnglishDialectFlags::default(),
                        german: GermanDialectFlags::default(),
                        portuguese: PortugueseDialectFlags::BRAZILIAN,
                        slovak: SlovakDialectFlags::default(),
                    }),
                    "AFRICAN" => Ok(ScopedDialectFlagsSerde {
                        english: EnglishDialectFlags::default(),
                        german: GermanDialectFlags::default(),
                        portuguese: PortugueseDialectFlags::AFRICAN,
                        slovak: SlovakDialectFlags::default(),
                    }),
                    _ => Err(Error::custom(format!("Unknown dialect string: {s}"))),
                }
            }
            _ => Err(Error::custom("Expected object or string for dialect flags")),
        }
    }
}
