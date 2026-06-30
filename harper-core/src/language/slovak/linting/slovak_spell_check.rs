//! Slovak spell check linter.
//!
//! This module provides the Slovak spell checking linter that identifies
//! misspelled words in Slovak text.

use crate::language::slovak::dialects::SlovakDialect;
use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{TokenStringExt, document::Document, spell::Dictionary};

/// A spell checker for Slovak text.
pub struct SlovakSpellCheck<T>
where
    T: Dictionary,
{
    dictionary: T,
    dialect: SlovakDialect,
}

impl<T: Dictionary> SlovakSpellCheck<T> {
    pub fn new(dictionary: T, dialect: SlovakDialect) -> Self {
        Self {
            dictionary,
            dialect,
        }
    }

    /// Get the dialect used by this spell checker.
    pub fn dialect(&self) -> SlovakDialect {
        self.dialect
    }

    /// Get spelling suggestions for a word using fuzzy matching.
    fn get_suggestions(&self, word: &[char]) -> Vec<Vec<char>> {
        // Use the dictionary's fuzzy matching (FST-based Levenshtein)
        let results = self.dictionary.fuzzy_match(word, 2, 5);

        // Extract suggestions from results
        results.into_iter().map(|r| r.word.to_vec()).collect()
    }
}

impl<T: Dictionary> Linter for SlovakSpellCheck<T> {
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
                            .map(Suggestion::ReplaceWith)
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
        "Checks for spelling errors in Slovak text"
    }
}