use std::sync::Arc;

use crate::linting::{LintKind, Suggestion};
use crate::spell::{Dictionary, FstDictionary, TrieDictionary};
use crate::{Document, TokenStringExt};

use super::{Lint, Linter};

pub struct SplitWords {
    dict: TrieDictionary<Arc<FstDictionary>>,
}

impl SplitWords {
    pub fn new() -> Self {
        Self {
            dict: TrieDictionary::new(FstDictionary::curated()),
        }
    }
}

impl Default for SplitWords {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter for SplitWords {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for word in document.iter_words() {
            // If it's a recognized word, we don't care about it.
            if let Some(_) = word.kind.as_word().unwrap() {
                continue;
            }

            let chars = document.get_span_content(&word.span);
            // The word that starts the compound
            let candidates = self.dict.find_words_with_common_prefix(chars);

            for candidate in candidates {
                if candidate.len() >= chars.len() {
                    continue;
                }

                // The potential word that completes the compound
                let remainder = &chars[candidate.len()..];
                if self.dict.contains_word(&remainder) {
                    let candidate_chars = candidate.as_ref();
                    let mut suggestion = Vec::new();

                    suggestion.extend(candidate_chars.iter());
                    suggestion.push(' ');
                    suggestion.extend(remainder.iter());

                    let original_word: String = chars.iter().collect();
                    let candidate_word: String = candidate_chars.iter().collect();
                    let remainder_word: String = remainder.iter().collect();

                    lints.push(Lint {
                        span: word.span,
                        lint_kind: LintKind::Typo,
                        suggestions: vec![Suggestion::ReplaceWith(suggestion)],
                        message: format!(
                            "`{original_word}` should probably be written as `{candidate_word} {remainder_word}`."
                        ),
                        ..Default::default()
                    });
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Finds missing spaces in improper compound words."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::SplitWords;

    #[test]
    fn issue_1905() {
        assert_suggestion_result(
            "I want to try this insteadof that.",
            SplitWords::default(),
            "I want to try this instead of that.",
        );
    }

    /// Same as above, but with the longer component word at the end.
    #[test]
    fn issue_1905_rev() {
        assert_suggestion_result(
            "I want to try thisinstead of that.",
            SplitWords::default(),
            "I want to try this instead of that.",
        );
    }

    #[test]
    fn split_common() {
        assert_suggestion_result(
            "This is notnot a problem.",
            SplitWords::default(),
            "This is not not a problem.",
        );
    }

    #[test]
    fn splits_multiple_compound_words() {
        assert_suggestion_result(
            "We stared intothe darkness and kindof panicked about sortof everything.",
            SplitWords::default(),
            "We stared into the darkness and kind of panicked about sort of everything.",
        );
    }

    #[test]
    fn splits_word_with_longer_prefix() {
        assert_suggestion_result(
            "The astronauts waited on the landingpad for hours.",
            SplitWords::default(),
            "The astronauts waited on the landing pad for hours.",
        );
    }

    #[test]
    fn splits_before_punctuation() {
        assert_suggestion_result(
            "This was kindof, actually, hilarious.",
            SplitWords::default(),
            "This was kind of, actually, hilarious.",
        );
    }

    #[test]
    fn ignores_known_compound_words() {
        assert_no_lints("Someone left early.", SplitWords::default());
    }

    #[test]
    fn ignores_prefix_without_valid_remainder() {
        assert_no_lints("The monkeyxyz escaped unnoticed.", SplitWords::default());
    }
}
