//! English language detector.
//!
//! This module provides the EnglishDetector struct that implements the LanguageDetector
//! trait for the language module system, using the original English detection functions
//! from language_detection.rs to match the master branch structure.

use crate::language::languages::Language;
use crate::language_detection::is_likely_english;
use crate::spell::FstDictionary;
use crate::Token;

/// English language detector (fallback).
#[derive(Debug)]
pub struct EnglishDetector;

impl crate::language::module::LanguageDetector for EnglishDetector {
    fn name(&self) -> &str {
        "english"
    }

    fn detect(&self, toks: &[Token], source: &[char], dict: &FstDictionary) -> Option<Language> {
        // Use Harper's built-in English detection from the root module
        let is_english = is_likely_english(toks, source, dict);

        if is_english {
            // Return American English as the detected dialect
            Some(Language::English(crate::Dialect::American))
        } else {
            None
        }
    }

    fn confidence(&self) -> f64 {
        // Lower confidence - used as fallback
        0.3
    }
}
