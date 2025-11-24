use super::dialect_trait::{Dialect, DialectFlags};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::{EnumCount as _, VariantArray as _};
use strum_macros::{Display, EnumCount, EnumIter, EnumString, VariantArray};

use std::convert::TryFrom;

use crate::{Document, TokenKind, TokenStringExt};

/// A regional dialect.
///
/// Note: these have bit-shifted values so that they can ergonomically integrate with
/// `DialectFlags`. Each value here must have a unique bit index inside
/// `DialectsUnderlyingType`.
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
    VariantArray,
)]
pub enum PortugueseDialect {
    European = 1 << 0,
    #[default]
    Brazilian = 1 << 1,
    African = 1 << 2,
}
impl Dialect for PortugueseDialect {
    type Flags = PortugueseDialectFlags;

    /// Tries to guess the dialect used in the document by finding which dialect is used the most.
    /// Returns `None` if it fails to find a single dialect that is used the most.
    #[allow(refining_impl_trait_internal)]
    fn try_guess_from_document(document: &Document) -> Option<Self> {
        Self::try_from(PortugueseDialectFlags::get_most_used_dialects_from_document(document)).ok()
    }

    /// Tries to get a dialect from its abbreviation. Returns `None` if the abbreviation is not
    /// recognized.
    ///
    /// # Examples
    ///
    /// ```
    /// use harper_core::PortugueseDialect;
    /// use harper_core::Dialect;
    ///
    /// let abbrs = ["PT", "BR", "AF"];
    /// let mut dialects = abbrs.iter().map(|abbr| PortugueseDialect::try_from_abbr(abbr));
    ///
    /// assert_eq!(Some(PortugueseDialect::European), dialects.next().unwrap()); // US
    /// assert_eq!(Some(PortugueseDialect::Brazilian), dialects.next().unwrap()); // CA
    /// assert_eq!(Some(PortugueseDialect::African), dialects.next().unwrap()); // AU
    /// ```
    #[allow(refining_impl_trait_internal)]
    fn try_from_abbr(abbr: &str) -> Option<Self> {
        match abbr {
            "PT" => Some(Self::European),
            "BR" => Some(Self::Brazilian),
            "AF" => Some(Self::African),
            _ => None,
        }
    }
}
impl TryFrom<PortugueseDialectFlags> for PortugueseDialect {
    type Error = ();

    /// Attempts to convert `DialectFlags` to a single `Dialect`.
    ///
    /// # Errors
    ///
    /// Will return `Err` if more than one dialect is enabled or if an undefined dialect is
    /// enabled.
    fn try_from(dialect_flags: PortugueseDialectFlags) -> Result<Self, Self::Error> {
        // Ensure only one dialect is enabled before converting.
        if dialect_flags.bits().count_ones() == 1 {
            match dialect_flags {
                df if df.is_dialect_enabled_strict(PortugueseDialect::European) => {
                    Ok(PortugueseDialect::European)
                }
                df if df.is_dialect_enabled_strict(PortugueseDialect::Brazilian) => {
                    Ok(PortugueseDialect::Brazilian)
                }
                df if df.is_dialect_enabled_strict(PortugueseDialect::African) => {
                    Ok(PortugueseDialect::African)
                }
                _ => Err(()),
            }
        } else {
            // More than one dialect enabled; can't soundly convert.
            Err(())
        }
    }
}

// The underlying type used for DialectFlags.
// At the time of writing, this is currently a `u8`. If we want to define more than 8 dialects in
// the future, we will need to switch this to a larger type.
type DialectFlagsUnderlyingType = u8;

bitflags::bitflags! {
    /// A collection of bit flags used to represent enabled dialects.
    ///
    /// This is generally used to allow a word (or similar) to be tagged with multiple dialects.
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
    #[serde(transparent)]
    pub struct PortugueseDialectFlags: DialectFlagsUnderlyingType {
        const EUROPEAN = PortugueseDialect::European as DialectFlagsUnderlyingType;
        const BRAZILIAN = PortugueseDialect::Brazilian as DialectFlagsUnderlyingType;
        const AFRICAN = PortugueseDialect::African as DialectFlagsUnderlyingType;
    }
}
impl DialectFlags<PortugueseDialect> for PortugueseDialectFlags {
    fn is_dialect_enabled(&self, dialect: PortugueseDialect) -> bool {
        self.is_empty() || self.intersects(Self::from_dialect(dialect))
    }

    /// Checks if the provided dialect is ***explicitly*** enabled.
    ///
    /// Unlike `is_dialect_enabled`, this will return false when no dialects are explicitly
    /// enabled.
    fn is_dialect_enabled_strict(&self, dialect: PortugueseDialect) -> bool {
        self.intersects(Self::from_dialect(dialect))
    }

    /// Constructs a `DialectFlags` from the provided `Dialect`, with only that dialect being
    /// enabled.
    ///
    /// # Panics
    ///
    /// This will panic if `dialect` represents a dialect that is not defined in
    /// `DialectFlags`.
    fn from_dialect(dialect: PortugueseDialect) -> Self {
        let Some(out) = Self::from_bits(dialect as DialectFlagsUnderlyingType) else {
            panic!("The '{dialect}' dialect isn't defined in DialectFlags!");
        };
        out
    }

    /// Gets the most commonly used dialect(s) in the document.
    ///
    /// If multiple dialects are used equally often, they will all be enabled in the returned
    /// `DialectFlags`. On the other hand, if there is a single dialect that is used the most, it
    /// will be the only one enabled.
    fn get_most_used_dialects_from_document(document: &Document) -> Self {
        // Initialize counters.
        let mut dialect_counters: [(PortugueseDialect, usize); PortugueseDialect::COUNT] =
            PortugueseDialect::VARIANTS
                .iter()
                .map(|d| (*d, 0))
                .collect_array()
                .unwrap();

        // Count word dialects.
        document.iter_words().for_each(|w| {
            if let TokenKind::Word(Some(lexeme_metadata)) = &w.kind {
                // If the token is a word, iterate though the dialects in `dialect_counters` and
                // increment those counters where the word has the respective dialect enabled.
                dialect_counters.iter_mut().for_each(|(dialect, count)| {
                    if lexeme_metadata.dialects.is_dialect_enabled(dialect.into()) {
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
            .fold(PortugueseDialectFlags::empty(), |acc, dialect| {
                // Fold most used dialects into `DialectFlags` via bitwise or.
                acc | Self::from_dialect(dialect.0)
            })
    }
}
impl Default for PortugueseDialectFlags {
    /// A default value with no dialects explicitly enabled.
    /// Implicitly, this state corresponds to all dialects being enabled.
    fn default() -> Self {
        Self::empty()
    }
}
