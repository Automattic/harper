//! English dialects.
//!
//! English dialect support for the Harper language system.

use serde::{Deserialize, Serialize};
use std::hash::Hash;

use crate::Document;
use crate::TokenKind;
use crate::language::dialects::dialect_trait::{Dialect, DialectFlags};
use crate::language::languages::LanguageFamily;
use crate::token_string_ext::TokenStringExt;

use strum::{EnumCount as _, VariantArray as _};
use strum_macros::{Display, EnumCount, EnumIter, EnumString, VariantArray};

/// The underlying type used for dialect flags.
type DialectFlagsUnderlyingType = u8;

/// English dialects.
#[derive(
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
    VariantArray,
    Default,
)]
pub enum EnglishDialect {
    #[default]
    American = 1 << 0,
    Canadian = 1 << 1,
    Australian = 1 << 2,
    British = 1 << 3,
    Indian = 1 << 4,
}

impl Dialect for EnglishDialect {
    type Flags = EnglishDialectFlags;

    /// Tries to guess the dialect used in the document by finding which dialect is used the most.
    /// Returns `None` if it fails to find a single dialect that is used the most.
    fn try_guess_from_document(document: &Document) -> Option<Self> {
        Self::try_from(EnglishDialectFlags::get_most_used_dialects_from_document(
            document,
        ))
        .ok()
    }

    /// Tries to get a dialect from its abbreviation. Returns `None` if the abbreviation is not
    /// recognized.
    fn try_from_abbr(abbr: &str) -> Option<Self> {
        match abbr {
            "US" => Some(Self::American),
            "CA" => Some(Self::Canadian),
            "AU" => Some(Self::Australian),
            "GB" => Some(Self::British),
            "IN" => Some(Self::Indian),
            _ => None,
        }
    }
}

bitflags::bitflags! {
    /// A collection of bit flags used to represent enabled dialects.
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
    #[serde(transparent)]
    pub struct EnglishDialectFlags: DialectFlagsUnderlyingType {
        const AMERICAN = EnglishDialect::American as DialectFlagsUnderlyingType;
        const CANADIAN = EnglishDialect::Canadian as DialectFlagsUnderlyingType;
        const AUSTRALIAN = EnglishDialect::Australian as DialectFlagsUnderlyingType;
        const BRITISH = EnglishDialect::British as DialectFlagsUnderlyingType;
        const INDIAN = EnglishDialect::Indian as DialectFlagsUnderlyingType;
    }
}

impl Default for EnglishDialectFlags {
    fn default() -> Self {
        Self::empty()
    }
}

impl DialectFlags<EnglishDialect> for EnglishDialectFlags {
    /// Checks if the provided dialect is enabled.
    /// If no dialect is explicitly enabled, it is assumed that all dialects are enabled.
    fn is_dialect_enabled(&self, dialect: EnglishDialect) -> bool {
        self.is_empty() || self.intersects(Self::from_dialect(dialect))
    }

    /// Checks if the provided dialect is ***explicitly*** enabled.
    fn is_dialect_enabled_strict(&self, dialect: EnglishDialect) -> bool {
        self.intersects(Self::from_dialect(dialect))
    }

    /// Constructs a `EnglishDialectFlags` from the provided `EnglishDialect`, with only that dialect being
    /// enabled.
    fn from_dialect(dialect: EnglishDialect) -> Self {
        let Some(out) = Self::from_bits(dialect as DialectFlagsUnderlyingType) else {
            panic!("The '{dialect}' dialect isn't defined in EnglishDialectFlags!");
        };
        out
    }

    /// Gets the most commonly used dialect(s) in the document.
    fn get_most_used_dialects_from_document(document: &Document) -> Self {
        // Initialize counters.
        let mut dialect_counters: [(EnglishDialect, usize); EnglishDialect::COUNT] =
            core::array::from_fn(|i| {
                let dialect = EnglishDialect::VARIANTS[i];
                (dialect, 0)
            });

        // Count word dialects.
        document.iter_words().for_each(|w| {
            if let TokenKind::Word(Some(lexeme_metadata)) = &w.kind {
                dialect_counters.iter_mut().for_each(|(dialect, count)| {
                    if lexeme_metadata
                        .dialects
                        .is_english_dialect_enabled(*dialect)
                    {
                        *count += 1;
                    }
                });
            }
        });

        // Find the maximum count.
        let max_count = dialect_counters
            .iter()
            .map(|(_, count)| *count)
            .max()
            .unwrap_or(0);

        // Collect all dialects with the maximum count.
        let mut result = Self::empty();
        for (dialect, count) in dialect_counters {
            if count == max_count && max_count > 0 {
                result.insert(Self::from_dialect(dialect));
            }
        }

        // If no dialects were found, return all dialects enabled (default behavior).
        if result.is_empty() {
            Self::all()
        } else {
            result
        }
    }

    fn get_most_used_dialects_from_document_language(
        document: &Document,
        _language: LanguageFamily,
    ) -> Self {
        // For English, we don't need language-specific filtering
        Self::get_most_used_dialects_from_document(document)
    }
}

impl TryFrom<EnglishDialectFlags> for EnglishDialect {
    type Error = ();

    /// Attempts to convert `DialectFlags` to a single `Dialect`.
    fn try_from(dialect_flags: EnglishDialectFlags) -> Result<Self, Self::Error> {
        // Ensure only one dialect is enabled before converting.
        if dialect_flags.bits().count_ones() == 1 {
            match dialect_flags {
                df if df.is_dialect_enabled_strict(EnglishDialect::American) => {
                    Ok(EnglishDialect::American)
                }
                df if df.is_dialect_enabled_strict(EnglishDialect::Canadian) => {
                    Ok(EnglishDialect::Canadian)
                }
                df if df.is_dialect_enabled_strict(EnglishDialect::Australian) => {
                    Ok(EnglishDialect::Australian)
                }
                df if df.is_dialect_enabled_strict(EnglishDialect::British) => {
                    Ok(EnglishDialect::British)
                }
                df if df.is_dialect_enabled_strict(EnglishDialect::Indian) => {
                    Ok(EnglishDialect::Indian)
                }
                _ => Err(()),
            }
        } else {
            // More than one dialect enabled; can't soundly convert.
            Err(())
        }
    }
}
