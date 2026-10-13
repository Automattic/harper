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
        let suggestions = results.into_iter().map(|r| r.word.to_vec()).collect();

        match_capitalization(word, suggestions)
    }
}

/// Carry the misspelled word's capitalization over to the suggestions.
///
/// The dictionary stores most words in lowercase, so without this a word
/// capitalized at the start of a sentence ("Nezavisle") would be corrected to
/// a lowercase word ("nezávisle"). An all-uppercase word ("NEZAVISLE") gets
/// all-uppercase suggestions. A suggestion with a capital letter after its
/// first character has its own fixed capitalization and is left alone, as in
/// the English spell check. Suggestions that become identical are merged.
fn match_capitalization(word: &[char], suggestions: Vec<Vec<char>>) -> Vec<Vec<char>> {
    let Some(first) = word.first() else {
        return suggestions;
    };

    let all_uppercase = word.len() > 1 && word.iter().all(|c| !c.is_lowercase());
    if !first.is_uppercase() {
        return suggestions;
    }

    let mut out: Vec<Vec<char>> = Vec::with_capacity(suggestions.len());
    for mut suggestion in suggestions {
        let has_internal_caps = suggestion.iter().skip(1).any(|c| c.is_uppercase());
        if all_uppercase {
            suggestion = suggestion.iter().flat_map(|c| c.to_uppercase()).collect();
        } else if !has_internal_caps && let Some(f) = suggestion.first_mut() {
            *f = f.to_uppercase().next().unwrap_or(*f);
        }
        if !out.contains(&suggestion) {
            out.push(suggestion);
        }
    }
    out
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

#[cfg(test)]
mod tests {
    use super::{SlovakSpellCheck, match_capitalization};
    use crate::language::slovak::dialects::SlovakDialect;
    use crate::language::slovak::spell::curated_slovak_dictionary;

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    fn strings(words: Vec<Vec<char>>) -> Vec<String> {
        words.into_iter().map(|w| w.into_iter().collect()).collect()
    }

    fn suggestions_for(word: &str) -> Vec<String> {
        let check = SlovakSpellCheck::new(curated_slovak_dictionary(), SlovakDialect::default());
        strings(check.get_suggestions(&chars(word)))
    }

    #[test]
    fn lowercase_typo_keeps_lowercase_suggestions() {
        let suggestions = suggestions_for("nezavisle");
        assert!(
            suggestions.contains(&"nezávisle".to_string()),
            "got {suggestions:?}"
        );
    }

    #[test]
    fn capitalized_typo_gets_capitalized_suggestions() {
        let suggestions = suggestions_for("Nezavisle");
        assert!(
            suggestions.contains(&"Nezávisle".to_string()),
            "got {suggestions:?}"
        );
        assert!(
            suggestions
                .iter()
                .all(|s| s.starts_with(char::is_uppercase)),
            "got {suggestions:?}"
        );
    }

    #[test]
    fn uppercase_typo_gets_uppercase_suggestions() {
        let suggestions = suggestions_for("NEZAVISLE");
        assert!(
            suggestions.contains(&"NEZÁVISLE".to_string()),
            "got {suggestions:?}"
        );
    }

    #[test]
    fn suggestion_with_internal_capital_is_left_alone() {
        let out = match_capitalization(&chars("Ofz"), vec![chars("OFZ"), chars("iPhone")]);
        assert_eq!(strings(out), vec!["OFZ", "iPhone"]);
    }

    #[test]
    fn suggestions_that_become_equal_are_merged() {
        let out = match_capitalization(
            &chars("Bratislva"),
            vec![chars("Bratislava"), chars("bratislava")],
        );
        assert_eq!(strings(out), vec!["Bratislava"]);
    }

    #[test]
    fn single_capital_letter_is_not_treated_as_all_uppercase() {
        let out = match_capitalization(&chars("A"), vec![chars("ab")]);
        assert_eq!(strings(out), vec!["Ab"]);
    }
}
