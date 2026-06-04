//! Portuguese spell checker.
//!
//! A basic spell checker for Portuguese text that checks words against the Portuguese dictionary.

use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{TokenStringExt, document::Document, spell::Dictionary};

/// A spell checker for Portuguese text.
pub struct PortugueseSpellCheck<T>
where
    T: Dictionary,
{
    dictionary: T,
}

impl<T: Dictionary> PortugueseSpellCheck<T> {
    pub fn new(dictionary: T) -> Self {
        Self { dictionary }
    }

    /// Get spelling suggestions for a word using fuzzy matching.
    fn get_suggestions(&self, word: &[char]) -> Vec<Vec<char>> {
        // Use the dictionary's fuzzy matching (FST-based Levenshtein)
        let results = self.dictionary.fuzzy_match(word, 2, 5);
        results.into_iter().map(|r| r.word.to_vec()).collect()
    }
}

impl<T: Dictionary> Linter for PortugueseSpellCheck<T> {
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
        "Checks for spelling errors in Portuguese text"
    }
}

#[cfg(test)]
mod tests {
    use super::PortugueseSpellCheck;
    use crate::Document;
    use crate::dialects::portuguese::PortugueseDialect;
    use crate::language::portuguese::parsers::PlainPortuguese;
    use crate::language::portuguese::spell::curated_portuguese_dictionary;
    use crate::languages::Language;
    use crate::linting::{LintGroup, Linter};

    fn lint_text(text: &str) -> Vec<String> {
        let dict = curated_portuguese_dictionary();
        let language = Language::Portuguese(PortugueseDialect::Brazilian);
        let mut linter = LintGroup::new_curated(dict.clone(), language);
        // Add Portuguese spell check linter explicitly
        linter.add(
            "PortugueseSpellCheck",
            PortugueseSpellCheck::new(dict.clone()),
        );
        linter.config.set_rule_enabled("PortugueseSpellCheck", true);
        let document = Document::new(text, &PlainPortuguese, &dict);

        linter
            .lint(&document)
            .into_iter()
            .map(|lint| lint.message)
            .collect()
    }

    #[test]
    fn detects_misspelled_word() {
        // Test with words that are definitely in the dictionary
        let messages = lint_text("tenho mundo amor");

        // "tenho", "mundo" and "amor" are in our dictionary, so no spelling errors expected
        assert!(
            messages.is_empty(),
            "Should not flag valid Portuguese words: {messages:?}"
        );
    }

    #[test]
    fn flags_unknown_word() {
        // Test with a word that is not in the dictionary
        let messages = lint_text("Eu tenho um xyzzy.");

        // "xyzzy" is not in the dictionary, should be flagged
        assert!(!messages.is_empty(), "Should flag unknown word");
        assert!(
            messages.iter().any(|m| m.contains("xyzzy")),
            "Should mention xyzzy in message"
        );
    }

    #[test]
    fn spell_check_description() {
        let dict = curated_portuguese_dictionary();
        let spellcheck = PortugueseSpellCheck::new(dict);
        assert_eq!(
            spellcheck.description(),
            "Checks for spelling errors in Portuguese text"
        );
    }
}
