use super::Suggestion;
use super::{Lint, LintKind, Linter};
use crate::document::Document;
use crate::spell::Dictionary;
use crate::TokenStringExt;

/// A linter that checks to make sure German nouns are capitalized.
/// In German, all nouns must be capitalized (not just proper nouns like in English).
pub struct GermanNounCapitalization<T>
where
    T: Dictionary,
{
    dictionary: T,
    // Common German noun suffixes to identify potential nouns
    noun_suffixes: Vec<Vec<char>>,
}

impl<T: Dictionary> GermanNounCapitalization<T> {
    pub fn new(dictionary: T) -> Self {
        // Common German noun suffixes (from LanguageTool and linguistic resources)
        let noun_suffixes = vec![
            vec!['h', 'e', 'i', 't'],     // -heit
            vec!['k', 'e', 'i', 't'],     // -keit
            vec!['u', 'n', 'g'],          // -ung
            vec!['i', 'e'],               // -ie (abstract nouns)
            vec!['i', 'k'],               // -ik
            vec!['i', 'o', 'n'],          // -ion
            vec!['t', 'ä', 't'],          // -tät
            vec!['s', 'c', 'h', 'a', 'f', 't'], // -schaft
        ];

        Self {
            dictionary,
            noun_suffixes,
        }
    }

    /// Check if a word is likely a German noun based on various heuristics
    fn is_likely_noun(&self, word: &[char], _document: &Document) -> bool {
        // Check if word is in dictionary and marked as noun
        if let Some(metadata) = self.dictionary.get_word_metadata(word) {
            if metadata.noun.is_some() {
                return true;
            }
        }

        // Check for common noun suffixes
        for suffix in &self.noun_suffixes {
            if word.len() > suffix.len() {
                let word_suffix = &word[word.len() - suffix.len()..];
                if word_suffix == *suffix {
                    return true;
                }
            }
        }

        false
    }
}

impl<T: Dictionary> Linter for GermanNounCapitalization<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for paragraph in document.iter_paragraphs() {
            for sentence in paragraph.iter_sentences() {
                // Get the first word of this sentence to skip it
                let first_word = sentence.first_non_whitespace();

                for word in sentence.iter_words() {
                    // Skip first word of sentence (handled by sentence capitalization)
                    if let Some(fw) = &first_word {
                        if word.span == fw.span {
                            continue;
                        }
                    }

                    let word_chars = document.get_span_content(&word.span);

                    // Skip words that are already capitalized
                    if let Some(first_char) = word_chars.first() {
                        if first_char.is_uppercase() {
                            continue;
                        }
                    }

                    // Skip non-alphabetic words
                    if word_chars.iter().all(|c| c.is_alphabetic()) {
                        // Check if this is a German noun that should be capitalized
                        if self.is_likely_noun(&word_chars, document) {
                            let mut replacement: Vec<char> = word_chars.to_vec();
                            if let Some(first_char) = replacement.first_mut() {
                                *first_char = first_char.to_uppercase().next().unwrap_or(*first_char);
                            }

                            lints.push(Lint {
                                span: word.span,
                                lint_kind: LintKind::Capitalization,
                                suggestions: vec![Suggestion::ReplaceWith(replacement)],
                                priority: 25, // High priority for German
                                message: format!(
                                    "In German, all nouns must be capitalized. \"{}\" appears to be a noun.",
                                    word_chars.iter().collect::<String>()
                                ),
                            });
                        }
                    }
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Ensures German nouns are properly capitalized"
    }
}