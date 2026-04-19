use super::Suggestion;
use super::{Lint, LintKind, Linter};
use crate::TokenStringExt;
use crate::document::Document;
use crate::spell::Dictionary;

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

    /// Try to check if a word is a valid German compound word.
    /// German freely combines nouns: "Haustür" = "Haus" + "Tür".
    /// Also handles Fugen-s: "Frühstücksspeck" = "Frühstück" + "s" + "Speck".
    fn try_compound_word_check(&self, word: &[char]) -> bool {
        if word.len() < 6 {
            return false;
        }

        for split_pos in 3..word.len() - 2 {
            let first_part = &word[..split_pos];
            let second_part = &word[split_pos..];

            // Direct split: "Haustür" → "Haus" + "Tür"
            if self.dictionary.contains_word(first_part)
                && self.dictionary.contains_word(second_part)
            {
                return true;
            }

            // Fugen-s: "Arbeitsstelle" → "Arbeit" + "s" + "Stelle"
            if second_part.first() == Some(&'s') && second_part.len() > 3 {
                let after_s = &second_part[1..];
                if self.dictionary.contains_word(first_part)
                    && self.dictionary.contains_word(after_s)
                {
                    return true;
                }
            }

            // Fugen-n: "Straßenrand" → "Straße" + "n" + "Rand"
            if second_part.first() == Some(&'n') && second_part.len() > 3 {
                let after_n = &second_part[1..];
                if self.dictionary.contains_word(first_part)
                    && self.dictionary.contains_word(after_n)
                {
                    return true;
                }
            }
        }

        false
    }

    /// Get spelling suggestions for a word using fuzzy matching.
    fn get_suggestions(&self, word: &[char]) -> Vec<Vec<char>> {
        // Use the dictionary's fuzzy matching (FST-based Levenshtein)
        let results = self.dictionary.fuzzy_match(word, 2, 5);
        let mut suggestions: Vec<Vec<char>> =
            results.into_iter().map(|r| r.word.to_vec()).collect();

        // Also try simple capitalization fix (common German error)
        if suggestions.is_empty() && word.len() > 1 {
            let mut capitalized = word.to_vec();
            if let Some(first_char) = capitalized.first_mut() {
                *first_char = first_char.to_uppercase().next().unwrap_or(*first_char);
            }
            if self.dictionary.contains_word(&capitalized) {
                suggestions.push(capitalized);
            }
        }

        suggestions.truncate(5);
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
                    let word_str: String = word_chars.iter().collect();

                    let message = if !suggestions.is_empty() {
                        let suggestions_str: Vec<String> = suggestions
                            .iter()
                            .map(|s| s.iter().collect::<String>())
                            .collect();
                        format!(
                            "Possible spelling error: \"{}\". Did you mean: {}?",
                            word_str,
                            suggestions_str.join(", ")
                        )
                    } else {
                        format!("Unknown word: \"{}\".", word_str)
                    };

                    lints.push(Lint {
                        span: word.span,
                        lint_kind: LintKind::Spelling,
                        suggestions: suggestions
                            .into_iter()
                            .map(|s| Suggestion::ReplaceWith(s))
                            .collect(),
                        priority: 20,
                        message,
                    });
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Checks for spelling errors in German text"
    }
}
