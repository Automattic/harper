//! English language detector.
//!
//! English language detection using Harper's built-in English detection functionality.

use crate::language::LanguageDetector;
use crate::language::english::dialects::EnglishDialect;
use crate::language::languages::Language;
use crate::spell::{Dictionary, FstDictionary};
use crate::{Token, TokenKind};

/// Check if given tokens are likely intended to represent English.
pub fn is_likely_english(toks: &[Token], source: &[char], dict: &impl Dictionary) -> bool {
    let mut total_words = 0;
    let mut valid_words = 0;
    let mut punctuation = 0;
    let mut unlintable = 0;

    for token in toks {
        match token.kind {
            TokenKind::Word(_) => {
                total_words += 1;

                let word_content = token.get_ch(source);
                if dict.contains_word(word_content) {
                    valid_words += 1;
                }
            }
            TokenKind::Punctuation(_) => punctuation += 1,
            TokenKind::Unlintable => unlintable += 1,
            _ => (),
        }
    }

    if total_words <= 7 && total_words - valid_words > 0 {
        return false;
    }

    if unlintable > valid_words {
        return false;
    }

    if (punctuation as f32 * 1.25) > valid_words as f32 {
        return false;
    }

    if (valid_words as f64 / total_words as f64) < 0.7 {
        return false;
    }

    true
}

/// English language detector (fallback).
#[derive(Debug)]
pub struct EnglishDetector;

impl LanguageDetector for EnglishDetector {
    fn name(&self) -> &str {
        "english"
    }

    fn detect(&self, toks: &[Token], source: &[char], dict: &FstDictionary) -> Option<Language> {
        // Use Harper's built-in English detection
        let is_english = is_likely_english(toks, source, dict);

        if is_english {
            // Return American English as the detected dialect
            Some(Language::English(EnglishDialect::American))
        } else {
            None
        }
    }

    fn confidence(&self) -> f64 {
        // Lower confidence - used as fallback
        0.3
    }
}

#[cfg(test)]
mod tests {
    use super::is_likely_english;
    use crate::Document;
    use crate::spell::FstDictionary;

    fn assert_not_english(source: &'static str) {
        let dict = FstDictionary::curated();
        let doc = Document::new_plain_english(source, &dict);
        let is_likely_english = is_likely_english(doc.get_tokens(), doc.get_source(), &dict);
        dbg!(source);
        assert!(!is_likely_english);
    }

    fn assert_is_english(source: &'static str) {
        let dict = FstDictionary::curated();
        let doc = Document::new_plain_english(source, &dict);
        let is_likely_english = is_likely_english(doc.get_tokens(), doc.get_source(), &dict);
        dbg!(source);
        assert!(is_likely_english);
    }

    #[test]
    fn test_non_english_detection() {
        // These should not be detected as English
        assert_not_english("asdf qwerty zxcv"); // Nonsense words
        assert_not_english("!!!! ???? ...."); // Mostly punctuation
        assert_not_english("xyz abc def"); // Short nonsense
    }

    #[test]
    fn test_english_detection() {
        // These should be detected as English
        assert_is_english(
            "This is a proper English sentence with enough words to pass the detection threshold.",
        );
        assert_is_english(
            "The quick brown fox jumps over the lazy dog and this should be detected as English.",
        );
    }
}
