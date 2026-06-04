//! German language detector.
//!
//! Uses characteristic German features:
//! - Special characters (ä, ö, ü, ß)
//! - Common German words and articles
//! - Low English word match rate

use crate::language::german::dialects::GermanDialect;
use crate::language_detection::LanguageDetector;
use crate::languages::Language;
use crate::spell::{Dictionary, FstDictionary};
use crate::{Token, TokenKind};

/// German language detector with high confidence due to unique characters.
#[derive(Debug)]
pub struct GermanDetector;

impl LanguageDetector for GermanDetector {
    fn name(&self) -> &str {
        "german"
    }

    fn detect(
        &self,
        toks: &[Token],
        source: &[char],
        dict: &FstDictionary,
        _default_language: Language,
    ) -> Option<Language> {
        let mut total_words = 0;
        let mut german_char_count = 0;
        let mut common_german_words = 0;
        let mut english_matches = 0;

        // High-confidence German indicators (articles, pronouns, common verbs)
        let german_indicators = [
            // Definite articles
            "der",
            "die",
            "das",
            "den",
            "dem",
            "des",
            // Indefinite articles
            "ein",
            "eine",
            "einer",
            "einen",
            "einem",
            "einen",
            // Personal pronouns
            "ich",
            "du",
            "er",
            "sie",
            "es",
            "wir",
            "ihr",
            "sie",
            "mich",
            "dich",
            "ihn",
            "sie",
            "es",
            "uns",
            "euch",
            // Verb forms
            "ist",
            "sind",
            "war",
            "waren",
            "hat",
            "habe",
            "haben",
            "hatte",
            "hatten",
            "werden",
            "wird",
            "wurde",
            "worden",
            "kann",
            "kannst",
            "kann",
            "können",
            "könnte",
            "machen",
            "macht",
            "machte",
            "machen",
            "gehen",
            "geht",
            "ging",
            "gingen",
            // Prepositions
            "im",
            "am",
            "um",
            "für",
            "durch",
            "während",
            "seit",
            "von",
            "zu",
            "bei",
            // Common words
            "das",
            "dass",
            "dies",
            "diese",
            "dieser",
            "dieses",
            "jener",
            "jene",
            "jenes",
            "nicht",
            "nichts",
            "kein",
            "keine",
            "keiner",
            "keinen",
            "nirgendwo",
            "auch",
            "noch",
            "schon",
            "nur",
            "doch",
            "ja",
            "nein",
            "oder",
            "und",
        ];

        for token in toks {
            match token.kind {
                TokenKind::Word(_) => {
                    total_words += 1;
                    let word_content: String = token.get_ch(source).iter().collect();

                    // Check for German special characters (very high confidence)
                    if word_content.contains('ä')
                        || word_content.contains('ö')
                        || word_content.contains('ü')
                        || word_content.contains('ß')
                    {
                        german_char_count += 1;
                    }

                    // Check for common German words
                    let lower_word = word_content.to_lowercase();
                    if german_indicators.contains(&lower_word.as_str()) {
                        common_german_words += 1;
                    }

                    // Check if in English dictionary
                    if dict.contains_word(token.get_ch(source)) {
                        english_matches += 1;
                    }
                }
                TokenKind::Unlintable => {}
                _ => {}
            }
        }

        // Need minimum words for reliable detection
        if total_words < 5 {
            return None;
        }

        // Calculate detection scores
        let german_char_ratio = german_char_count as f64 / total_words as f64;
        let german_word_ratio = common_german_words as f64 / total_words as f64;
        let english_match_ratio = if total_words > 0 {
            english_matches as f64 / total_words as f64
        } else {
            0.0
        };

        // High confidence: German special characters present
        if german_char_ratio >= 0.01 {
            // 1%+ words have ä, ö, ü, or ß
            return Some(Language::German(GermanDialect::Standard));
        }

        // Check if English is clearly dominant (more than 65% English words)
        if english_match_ratio >= 0.65 {
            return None; // English is clearly dominant
        }

        // Strong indicator: Many common German words
        if german_word_ratio >= 0.20 {
            // 20%+ words are common German words
            return Some(Language::German(GermanDialect::Standard));
        }

        // Medium confidence: Low English match but some German words
        if english_match_ratio < 0.4 && german_word_ratio >= 0.08 {
            return Some(Language::German(GermanDialect::Standard));
        }

        None
    }

    fn confidence(&self) -> f64 {
        // High confidence due to unique character detection
        0.95
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use crate::dialects::english::EnglishDialect;
    use crate::languages::Language;
    use crate::spell::FstDictionary;

    fn test_detection(text: &str, expected_german: bool) {
        let dict = FstDictionary::curated();
        let doc = Document::new_plain_english_curated(text);
        let detector = GermanDetector;

        let default_lang = Language::English(EnglishDialect::American);
        let result = detector.detect(doc.get_tokens(), doc.get_source(), &dict, default_lang);
        assert_eq!(result.is_some(), expected_german, "Failed for: {}", text);
        if expected_german {
            assert_eq!(result.unwrap(), Language::German(GermanDialect::Standard));
        }
    }

    #[test]
    fn detects_german_special_chars() {
        test_detection("Der Hund spielt im Garten mit Äpfeln und Ölkannen.", true);
    }

    #[test]
    fn detects_common_german_words() {
        test_detection(
            "Der Hund ist im Garten. Die Katze schläft auf dem Sofa. Das Auto ist sehr schnell.",
            true,
        );
    }

    #[test]
    fn detects_mixed_german_english() {
        test_detection(
            "Der Hund plays im Garten. die Katze sleeps auf dem Sofa.",
            true,
        );
    }

    #[test]
    fn rejects_english() {
        test_detection(
            "The dog plays in the garden. The cat sleeps on the sofa. The car is very fast.",
            false,
        );
    }

    #[test]
    fn rejects_short_text() {
        test_detection("Der Hund", false);
    }

    #[test]
    fn detects_longer_german_text() {
        test_detection(
            "Das ist eine Anleitung für den Gebrauch der Maschine. \
             Der Hund spielt im Garten und die Katze schläft auf dem Sofa. \
             Das Auto ist sehr schnell und der Vogel singt im Baum. \
             Wir gehen heute ins Kino und essen danach im Restaurant.",
            true,
        );
    }
}
