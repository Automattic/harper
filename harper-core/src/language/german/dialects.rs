//! German dialect support.

use crate::dialects::dialect_trait::{Dialect, DialectFlags};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use strum::{EnumCount as _, VariantArray as _};
use strum_macros::{Display, EnumCount, EnumIter, EnumString, VariantArray};

use crate::{Document, TokenKind, TokenStringExt};

/// German dialects supported by Harper.
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
    Default,
    EnumCount,
    EnumString,
    EnumIter,
    Display,
    VariantArray,
)]
pub enum GermanDialect {
    /// Standard German (Deutschland)
    #[default]
    Standard = 1 << 0,
    /// Austrian German (Österreich)
    Austrian = 1 << 1,
    /// Swiss German (Schweiz)
    Swiss = 1 << 2,
}

impl GermanDialect {
    /// Tries to get a dialect from its abbreviation.
    #[must_use]
    pub fn try_from_abbr(abbr: &str) -> Option<Self> {
        match abbr {
            "DE" | "Standard" => Some(Self::Standard),
            "AT" | "Austrian" => Some(Self::Austrian),
            "CH" | "Swiss" => Some(Self::Swiss),
            _ => None,
        }
    }
}

impl Dialect for GermanDialect {
    type Flags = GermanDialectFlags;

    /// Tries to guess the dialect used in the document by finding which dialect is used the most.
    /// Returns `None` if it fails to find a single dialect that is used the most.
    fn try_guess_from_document(document: &Document) -> Option<Self> {
        Self::try_from(GermanDialectFlags::get_most_used_dialects_from_document(
            document,
        ))
        .ok()
    }

    fn try_from_abbr(abbr: &str) -> Option<Self> {
        Self::try_from_abbr(abbr)
    }
}

impl TryFrom<GermanDialectFlags> for GermanDialect {
    type Error = ();

    /// Attempts to convert `DialectFlags` to a single `Dialect`.
    ///
    /// # Errors
    ///
    /// Will return `Err` if more than one dialect is enabled or if an undefined dialect is
    /// enabled.
    fn try_from(dialect_flags: GermanDialectFlags) -> Result<Self, Self::Error> {
        // Ensure only one dialect is enabled before converting.
        if dialect_flags.bits().count_ones() == 1 {
            if dialect_flags.is_dialect_enabled_strict(GermanDialect::Standard) {
                Ok(GermanDialect::Standard)
            } else if dialect_flags.is_dialect_enabled_strict(GermanDialect::Austrian) {
                Ok(GermanDialect::Austrian)
            } else if dialect_flags.is_dialect_enabled_strict(GermanDialect::Swiss) {
                Ok(GermanDialect::Swiss)
            } else {
                Err(())
            }
        } else {
            // More than one dialect enabled; can't soundly convert.
            Err(())
        }
    }
}

// The underlying type used for DialectFlags.
type DialectFlagsUnderlyingType = u8;

bitflags::bitflags! {
    /// A collection of bit flags used to represent enabled German dialects.
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
    #[serde(transparent)]
    pub struct GermanDialectFlags: DialectFlagsUnderlyingType {
        const STANDARD = GermanDialect::Standard as DialectFlagsUnderlyingType;
        const AUSTRIAN = GermanDialect::Austrian as DialectFlagsUnderlyingType;
        const SWISS = GermanDialect::Swiss as DialectFlagsUnderlyingType;
    }
}

impl DialectFlags<GermanDialect> for GermanDialectFlags {
    /// Checks if the provided dialect is enabled.
    /// If no dialect is explicitly enabled, it is assumed that all dialects are enabled.
    fn is_dialect_enabled(&self, dialect: GermanDialect) -> bool {
        self.is_empty() || self.intersects(Self::from_dialect(dialect))
    }

    /// Checks if the provided dialect is ***explicitly*** enabled.
    fn is_dialect_enabled_strict(&self, dialect: GermanDialect) -> bool {
        self.intersects(Self::from_dialect(dialect))
    }

    /// Constructs a `DialectFlags` from the provided `Dialect`.
    fn from_dialect(dialect: GermanDialect) -> Self {
        let Some(out) = Self::from_bits(dialect as DialectFlagsUnderlyingType) else {
            panic!("The '{dialect}' dialect isn't defined in DialectFlags!");
        };
        out
    }

    /// Gets the most commonly used dialect(s) in the document.
    fn get_most_used_dialects_from_document(document: &Document) -> Self {
        // Initialize counters.
        let mut dialect_counters: [(GermanDialect, usize); GermanDialect::COUNT] =
            GermanDialect::VARIANTS
                .iter()
                .map(|d| (*d, 0))
                .collect_array()
                .unwrap();

        // Count word dialects.
        document.iter_words().for_each(|w| {
            if let TokenKind::Word(Some(lexeme_metadata)) = &w.kind {
                dialect_counters.iter_mut().for_each(|(dialect, count)| {
                    if lexeme_metadata.dialects.is_dialect_enabled(dialect) {
                        *count += 1;
                    }
                });
            }
        });

        // Find max counter.
        let max_counter = dialect_counters
            .iter()
            .map(|(_, count)| count)
            .max()
            .unwrap();
        // Get and convert the collection of most used dialects into a `DialectFlags`.
        dialect_counters
            .into_iter()
            .filter(|(_, count)| count == max_counter)
            .fold(GermanDialectFlags::empty(), |acc, dialect| {
                acc | Self::from_dialect(dialect.0)
            })
    }
}

impl Default for GermanDialectFlags {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialect_abbreviations() {
        assert_eq!(
            GermanDialect::try_from_abbr("DE"),
            Some(GermanDialect::Standard)
        );
        assert_eq!(
            GermanDialect::try_from_abbr("AT"),
            Some(GermanDialect::Austrian)
        );
        assert_eq!(
            GermanDialect::try_from_abbr("CH"),
            Some(GermanDialect::Swiss)
        );
        assert_eq!(
            GermanDialect::try_from_abbr("Standard"),
            Some(GermanDialect::Standard)
        );
        assert_eq!(GermanDialect::try_from_abbr("XY"), None);
    }
}
