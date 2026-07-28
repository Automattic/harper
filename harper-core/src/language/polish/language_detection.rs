use crate::language::LanguageDetector;
use crate::language::languages::Language;
use crate::language::polish::dialects::PolishDialect;
use crate::spell::{Dictionary, FstDictionary};
use crate::{Token, TokenKind};

/// Polish language detector.
#[derive(Debug)]
pub struct PolishDetector;

impl LanguageDetector for PolishDetector {
    fn name(&self) -> &str {
        "polish"
    }

    fn detect(&self, toks: &[Token], source: &[char], dict: &FstDictionary) -> Option<Language> {
        let mut total_words = 0;
        let mut polish_char_count = 0;
        let mut common_polish_words = 0;
        let mut english_matches = 0;

        // Polish indicators - common words and articles
        let polish_indicators = [
            // Common words
            "i", "w", "na", "z", "do", "że", "jest", "nie", "się", "tak",
            // Articles/pronouns
            "to", "ten", "ta", "to", "ci", "ci", "te", "te", // Verbs
            "być", "mieć", "robić", "iść", "mówić", "pisać", "czytać", // Prepositions
            "po", "przed", "pod", "nad", "między", "bez", "dla",
        ];

        for token in toks {
            match token.kind {
                TokenKind::Word(_) => {
                    total_words += 1;
                    let word_content: String = token.get_ch(source).iter().collect();

                    // Check for Polish special characters (high confidence)
                    if word_content.contains('ą')
                        || word_content.contains('ć')
                        || word_content.contains('ę')
                        || word_content.contains('ł')
                        || word_content.contains('ń')
                        || word_content.contains('ó')
                        || word_content.contains('ś')
                        || word_content.contains('ź')
                        || word_content.contains('ż')
                    {
                        polish_char_count += 1;
                    }

                    // Check for common Polish words
                    let lower_word = word_content.to_lowercase();
                    if polish_indicators.contains(&lower_word.as_str()) {
                        common_polish_words += 1;
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
        let polish_char_ratio = polish_char_count as f64 / total_words as f64;
        let polish_word_ratio = common_polish_words as f64 / total_words as f64;
        let english_match_ratio = if total_words > 0 {
            english_matches as f64 / total_words as f64
        } else {
            0.0
        };

        // High confidence: Polish special characters present
        if polish_char_ratio >= 0.01 {
            return Some(Language::Polish(PolishDialect::Standard));
        }

        // Check if English is clearly dominant (more than 65% English words)
        if english_match_ratio >= 0.65 {
            return None; // English is clearly dominant
        }

        // Strong indicator: Many common Polish words
        if polish_word_ratio >= 0.20 {
            return Some(Language::Polish(PolishDialect::Standard));
        }

        // Medium confidence: Low English match but some Polish words
        if english_match_ratio < 0.4 && polish_word_ratio >= 0.08 {
            return Some(Language::Polish(PolishDialect::Standard));
        }

        None
    }

    fn confidence(&self) -> f64 {
        0.90
    }
}
