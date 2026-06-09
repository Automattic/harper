//! Language registry - re-exports from the language manifest.
//!
//! This module provides backward-compatible re-exports of language integration
//! functions. For new code, prefer importing directly from `crate::language::manifest`.

pub use crate::language::manifest::{
    ProseLanguage, add_language_specific_linters, dictionary, dictionary_for_language,
    parser_for_prose, prose_language,
};
