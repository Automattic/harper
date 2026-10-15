//! Slovak language support for Harper.
//!
//! This module contains all Slovak-specific functionality including:
//! - Spell checking with dictionary support
//! - Grammar rules and linters
//! - Parser for Slovak text
//! - Slovak dictionary
//! - Language detection

pub mod dialects;
pub mod language_detection;
pub mod lexing;
pub mod linting;
pub mod module;
pub mod parsers;
pub mod spell;

#[cfg(test)]
pub mod tests;
