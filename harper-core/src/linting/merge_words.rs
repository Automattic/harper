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

        for (i, (a, w, apostrophe, s)) in document.tokens().tuple_windows().enumerate() {
            if !a.kind.is_word()
                || !w.kind.is_space()
                || !apostrophe.kind.is_apostrophe()
                || !s.kind.is_word()
                || !matches!(document.get_span_content(&s.span), ['s' | 'S'])
                || document
                    .get_tokens()
                    .get(i + 4)
                    .is_some_and(|next| next.kind.is_apostrophe())
            {
                continue;
            }

            let a_chars = document.get_span_content(&a.span);
            merged_word.clear();
            merged_word.extend_from_slice(a_chars);
            merged_word.push('\'');
            merged_word.extend_from_slice(document.get_span_content(&s.span));

            if self.dict.contains_word(&merged_word) {
                merged_word[a_chars.len()] = document.get_span_content(&apostrophe.span)[0];
                lints.push(Lint {
                    span: Span::new(a.span.start, s.span.end),
                    lint_kind: LintKind::WordChoice,
                    suggestions: vec![Suggestion::ReplaceWith(merged_word.to_vec())],
                    message: "Remove the space before the apostrophe.".to_owned(),
                    priority: 31,
                });
            }
        }

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
    use crate::linting::tests::{
        assert_first_suggestion_result, assert_lint_count, assert_suggestion_result,
    };

    use super::MergeWords;

    #[test]
    fn merges_separated_possessive() {
        assert_first_suggestion_result(
            "The carrier 's family.",
            MergeWords::default(),
            "The carrier's family.",
        );
    }

    #[test]
    fn merges_proper_noun_possessive() {
        assert_first_suggestion_result(
            "Huntington 's disease.",
            MergeWords::default(),
            "Huntington's disease.",
        );
    }

    #[test]
    fn merges_separated_contraction() {
        assert_first_suggestion_result("That 's fine.", MergeWords::default(), "That's fine.");
    }

    #[test]
    fn merges_curly_apostrophe() {
        assert_first_suggestion_result("That ’s fine.", MergeWords::default(), "That’s fine.");
    }

    #[test]
    fn merges_multiple_spaces() {
        assert_first_suggestion_result("That   's fine.", MergeWords::default(), "That's fine.");
    }

    #[test]
    fn merges_tab() {
        assert_first_suggestion_result("That\t's fine.", MergeWords::default(), "That's fine.");
    }

    #[test]
    fn preserves_uppercase_suffix() {
        assert_first_suggestion_result("JOHN 'S car.", MergeWords::default(), "JOHN'S car.");
    }

    #[test]
    fn merges_multiple_suffixes() {
        assert_first_suggestion_result(
            "Huntington 's disease affects the carrier 's family.",
            MergeWords::default(),
            "Huntington's disease affects the carrier's family.",
        );
    }

    #[test]
    fn allows_quoted_letters() {
        for text in [
            "The letter 's' is lowercase.",
            "The letter ’s’ is lowercase.",
        ] {
            assert_lint_count(text, MergeWords::default(), 0);
        }
    }

    #[test]
    fn allows_joined_suffixes() {
        assert_lint_count("That's the carrier's family.", MergeWords::default(), 0);
    }

    #[test]
    fn allows_other_suffixes() {
        for text in ["We 're ready.", "I 'll leave.", "They 've finished."] {
            assert_lint_count(text, MergeWords::default(), 0);
        }
    }

    #[test]
    fn allows_line_break_before_quoted_s() {
        assert_lint_count("Huntington\n's", MergeWords::default(), 0);
    }

    #[test]
    fn allows_unrecognized_merged_word() {
        assert_lint_count("and 's", MergeWords::default(), 0);
    }

    #[test]
    fn curated_merge_precedes_suffix_spelling_and_following_word_merge() {
        let group = crate::linting::LintGroup::new_curated(
            crate::spell::FstDictionary::curated(),
            crate::Dialect::American,
        );
        assert_first_suggestion_result("It 's our family.", group, "It's our family.");
    }

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

    #[test]
    fn checks_weirdly() {
        assert_suggestion_result(
            "This is wrapped around the line weirdl y.",
            MergeWords::default(),
            "This is wrapped around the line weirdly.",
        );
    }
}
