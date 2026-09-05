//! German language support for Harper.
//!
//! This module contains all German-specific functionality including:
//! - Spell checking with compound word support
//! - Grammar rules and linters
//! - Parser for German text
//! - German dictionary

pub mod dialects;
pub mod language_detection;
pub mod lexing;
pub mod linting;
pub mod module;
pub mod parsers;
pub mod spell;

#[cfg(test)]
pub mod tests;
