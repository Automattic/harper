//! English dialects.
//!
//! This module provides type aliases to the original English dialect types
//! from dict_word_metadata.rs to maintain compatibility with the master branch
//! while allowing English to also be a LanguageModule.

use crate::dict_word_metadata::{Dialect, DialectFlags};

/// Type alias for the original English Dialect enum.
pub type EnglishDialect = Dialect;

/// Type alias for the original English DialectFlags bitflags.
pub type EnglishDialectFlags = DialectFlags;
