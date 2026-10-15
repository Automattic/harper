use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;
use strum::{EnumCount, IntoEnumIterator};

use crate::Document;
use crate::language::languages::LanguageFamily;

pub trait Dialect:
    Debug
    + Clone
    + Copy
    + Serialize
    + for<'de> Deserialize<'de>
    + PartialEq
    + PartialOrd
    + Eq
    + Hash
    + EnumCount
    + FromStr
    + IntoEnumIterator
    + Display
{
    type Flags: DialectFlags<Self>;
    /// Tries to guess the dialect used in the document by finding which dialect is used the most.
    /// Returns `None` if it fails to find a single dialect that is used the most.
    #[must_use]
    fn try_guess_from_document(document: &Document) -> Option<Self>;

    /// Tries to get a dialect from its abbreviation. Returns `None` if the abbreviation is not
    /// recognized.
    ///
    /// # Examples
    ///
    /// ```
    /// use harper_core::language::english::dialects::EnglishDialect;
    /// use harper_core::language::dialects::dialect_trait::Dialect;
    ///
    /// // EnglishDialect implements Dialect
    /// let abbrs = ["US", "CA", "AU", "GB"];
    /// let mut dialects = abbrs.iter().map(|abbr| EnglishDialect::try_from_abbr(abbr));
    ///
    /// assert_eq!(Some(EnglishDialect::American), dialects.next().unwrap()); // US
    /// assert_eq!(Some(EnglishDialect::Canadian), dialects.next().unwrap()); // CA
    /// assert_eq!(Some(EnglishDialect::Australian), dialects.next().unwrap()); // AU
    /// assert_eq!(Some(EnglishDialect::British), dialects.next().unwrap()); // GB
    /// ```
    #[must_use]
    fn try_from_abbr(abbr: &str) -> Option<Self>;
}

pub trait DialectFlags<ParentDialect>: Default
where
    ParentDialect: Dialect<Flags = Self>,
{
    /// Checks if the provided dialect is enabled.
    /// If no dialect is explicitly enabled, it is assumed that all dialects are enabled.
    #[must_use]
    fn is_dialect_enabled(&self, dialect: ParentDialect) -> bool;

    ///
    /// Checks if the provided dialect is ***explicitly*** enabled.
    ///
    /// Unlike `is_dialect_enabled`, this will return false when no dialects are explicitly
    /// enabled.
    #[must_use]
    fn is_dialect_enabled_strict(&self, dialect: ParentDialect) -> bool;

    /// Constructs a `DialectFlags` from the provided `Dialect`, with only that dialect being
    /// enabled.
    ///
    /// # Panics
    ///
    /// This will panic if `dialect` represents a dialect that is not defined in
    /// `DialectFlags`.
    #[must_use]
    fn from_dialect(dialect: ParentDialect) -> Self;

    /// Gets the most commonly used dialect(s) in the document.
    ///
    /// If multiple dialects are used equally often, they will all be enabled in the returned
    /// `DialectFlags`. On the other hand, if there is a single dialect that is used the most, it
    /// will be the only one enabled.
    ///
    /// **Override this only when the dictionary carries dialect metadata for the language.**
    /// Counting needs a per-word signal, and English is the only language that has one. The
    /// default says "I cannot tell" by enabling nothing, which
    /// `Dialect::try_guess_from_document` turns into `None`.
    ///
    /// Three languages previously wrote out the counting loop with the body commented out. That
    /// walks every word of every document to leave all the counters at zero, and a maximum over
    /// all-zero counters then matches *every* dialect -- so the stub returned "all dialects",
    /// which is the opposite of what its own comment claimed, and `try_from` rejected it only
    /// because more than one was set.
    #[must_use]
    fn get_most_used_dialects_from_document(_document: &Document) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    fn get_most_used_dialects_from_document_language(
        document: &Document,
        _language: LanguageFamily,
    ) -> Self {
        Self::get_most_used_dialects_from_document(document)
    }
}
