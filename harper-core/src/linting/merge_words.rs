use std::sync::Arc;

use itertools::Itertools;

use super::{Lint, LintKind, Linter, Suggestion};
use crate::spell::{Dictionary, FstDictionary};
use crate::{CharString, Document, Span};

pub struct MergeWords {
    dict: Arc<FstDictionary>,
}

impl MergeWords {
    pub fn new() -> Self {
        Self {
            dict: FstDictionary::curated(),
        }
    }
}

impl Default for MergeWords {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter for MergeWords {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        let mut merged_word = CharString::new();

        for (a, w, b) in document.tokens().tuple_windows() {
            if !a.kind.is_word() || !w.kind.is_whitespace() || !b.kind.is_word() {
                continue;
            }

            let a_chars = document.get_span_content(&a.span);
            let b_chars = document.get_span_content(&b.span);

            if (a_chars.len() == 1 && a_chars[0].is_uppercase())
                || (b_chars.len() == 1 && b_chars[0].is_uppercase())
            {
                continue;
            }

            // Not super helpful in this case, so we skip it
            if matches!(a_chars, ['a']) || matches!(b_chars, ['a']) {
                continue;
            }

            merged_word.clear();
            merged_word.extend_from_slice(a_chars);
            merged_word.extend_from_slice(b_chars);

            if self.dict.contains_word(&merged_word)
                && (!self.dict.contains_word(a_chars) || !self.dict.contains_word(b_chars))
            {
                lints.push(Lint {
                    span: Span::new(a.span.start, b.span.end),
                    lint_kind: LintKind::WordChoice,
                    suggestions: vec![Suggestion::ReplaceWith(merged_word.to_vec())],
                    message: "It seems these words would go better together.".to_owned(),
                    priority: 63,
                });
            }

            merged_word.clear();
            merged_word.extend_from_slice(a_chars);
            merged_word.push('\'');
            merged_word.extend_from_slice(b_chars);

            if self.dict.contains_word(&merged_word)
                && (!self.dict.contains_word(a_chars) || !self.dict.contains_word(b_chars))
            {
                lints.push(Lint {
                    span: Span::new(a.span.start, b.span.end),
                    lint_kind: LintKind::WordChoice,
                    suggestions: vec![Suggestion::ReplaceWith(merged_word.to_vec())],
                    message: "It seems you intended to make this a contraction.".to_owned(),
                    priority: 63,
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Accidentally inserting a space inside a word is common. This rule looks for valid words that are split by whitespace."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::MergeWords;

    #[test]
    fn clean() {
        assert_lint_count(
            "When referring to the political party, make sure to treat them as a proper noun.",
            MergeWords::default(),
            0,
        );
    }

    #[test]
    fn heretofore() {
        assert_lint_count(
            "This is a her etofore unseen problem.",
            MergeWords::default(),
            1,
        );
    }

    #[test]
    fn therefore() {
        assert_lint_count("The refore", MergeWords::default(), 1);
    }

    #[test]
    fn that_is_contraction() {
        assert_suggestion_result("That s", MergeWords::default(), "That's");
    }

    #[test]
    fn allows_issue_722() {
        assert_lint_count("Leaving S and K alone.", MergeWords::default(), 0);
        assert_lint_count("Similarly an S with a line.", MergeWords::default(), 0);
    }
}
