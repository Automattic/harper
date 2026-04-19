use super::Suggestion;
use super::{Lint, LintKind, Linter};
use crate::document::Document;
use crate::spell::Dictionary;
use crate::TokenStringExt;

/// A spell checker for German text with compound word handling.
pub struct GermanSpellCheck<T>
where
    T: Dictionary,
{
    dictionary: T,
}

impl<T: Dictionary> GermanSpellCheck<T> {
    pub fn new(dictionary: T) -> Self {
        Self { dictionary }
    }

    /// Try to check if a word is a valid German compound word
    fn try_compound_word_check(&self, word: &[char]) -> bool {
        // German compound words can be very long
        // Try splitting at common boundaries

        if word.len() < 10 {
            return false;
        }

        // Try various split positions
        for split_pos in 5..word.len() - 4 {
            let first_part = &word[..split_pos];
            let second_part = &word[split_pos..];

            if self.dictionary.contains_word(first_part) &&
               self.dictionary.contains_word(second_part) {
                // It's a valid compound word
                return true;
            }
        }

        false
    }

    /// Get spelling suggestions for a word
    fn get_suggestions(&self, word: &[char]) -> Vec<Vec<char>> {
        let mut suggestions = Vec::new();

        // For MVP, try simple variations
        // 1. Capitalize first letter (common error in German)
        if word.len() > 1 {
            let mut capitalized = word.to_vec();
            if let Some(first_char) = capitalized.first_mut() {
                *first_char = first_char.to_uppercase().next().unwrap_or(*first_char);
            }
            if self.dictionary.contains_word(&capitalized) {
                suggestions.push(capitalized);
            }
        }

        // 2. Lowercase first letter (if all caps)
        if word.len() > 1 && word.iter().all(|c| c.is_uppercase()) {
            let mut lowercase = word.to_vec();
            for c in &mut lowercase {
                if let Some(l) = c.to_lowercase().next() {
                    *c = l;
                }
            }
            if self.dictionary.contains_word(&lowercase) {
                suggestions.push(lowercase);
            }
        }

        // Limit suggestions
        suggestions.truncate(3);
        suggestions
    }
}

impl<T: Dictionary> Linter for GermanSpellCheck<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for paragraph in document.iter_paragraphs() {
            for sentence in paragraph.iter_sentences() {
                for word in sentence.iter_words() {
                    let word_chars = document.get_span_content(&word.span);

                    // Skip words in dictionary
                    if self.dictionary.contains_word(word_chars) {
                        continue;
                    }

                    // Try compound word splitting
                    if self.try_compound_word_check(word_chars) {
                        continue;
                    }

                    // Get spelling suggestions
                    let suggestions = self.get_suggestions(word_chars);

                    if !suggestions.is_empty() {
                        let word_str: String = word_chars.iter().collect();
                        let suggestions_str: Vec<String> = suggestions.iter()
                            .map(|s| s.iter().collect::<String>())
                            .collect();

                        lints.push(Lint {
                            span: word.span,
                            lint_kind: LintKind::Spelling,
                            suggestions: suggestions.into_iter()
                                .map(|s| Suggestion::ReplaceWith(s))
                                .collect(),
                            priority: 20,
                            message: format!(
                                "Possible spelling error: \"{}\". Did you mean: {}?",
                                word_str,
                                suggestions_str.join(", ")
                            ),
                        });
                    }
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Checks for spelling errors in German text"
    }
}