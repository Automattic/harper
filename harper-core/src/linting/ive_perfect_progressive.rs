use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::{Token, TokenKind};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct IvePerfectProgressive {
    expr: Box<dyn Expr>,
}

impl Default for IvePerfectProgressive {
    fn default() -> Self {
        let expr = SequenceExpr::word_set(&["I've", "We've", "You've", "They've"])
            .t_ws()
            .then_kind_both(TokenKind::is_verb, TokenKind::is_verb_progressive_form);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for IvePerfectProgressive {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let have_tok = toks.iter().find(|t| t.kind.is_word())?;
        let span = have_tok.span;

        // Determine correct progressive replacement for the matched contraction
        let matched_str: String = span.get_content(src).into_iter().collect();
        let lower = matched_str.to_lowercase();
        let progressive_replacement = if lower.starts_with("i'v") {
            "I'm"
        } else if lower.starts_with("we'v") {
            "We're"
        } else if lower.starts_with("you'v") {
            "You're"
        } else if lower.starts_with("they'v") {
            "They're"
        } else {
            // Fallback: default to replacing with present progressive of be for safety
            "I'm"
        };

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![
                Suggestion::replace_with_match_case(
                    progressive_replacement.chars().collect(),
                    span.get_content(src),
                ),
                Suggestion::InsertAfter(" been".chars().collect()),
            ],
            message: "Use present progressive (`…'re/…'m …`) or present perfect progressive (`…'ve been …`) instead of `…'ve …ing`.".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Detects the ungrammatical pattern `<pronoun>'ve` (e.g., `I've`, `We've`, `You've`, `They've`) followed directly by a gerund and suggests either the present progressive (e.g., `I'm/We're/You're/They're …`) or the present perfect progressive (e.g., `I've/We've/You've/They've been …`)."
    }
}

#[cfg(test)]
mod tests {
    use super::IvePerfectProgressive;
    use crate::linting::tests::{
        assert_good_and_bad_suggestions, assert_lint_count, assert_suggestion_result,
    };

    #[test]
    fn suggests_im_looking() {
        assert_suggestion_result(
            "I've looking into it.",
            IvePerfectProgressive::default(),
            "I'm looking into it.",
        );
    }

    #[test]
    fn corrects_basic_im() {
        assert_suggestion_result(
            "I've looking into it.",
            IvePerfectProgressive::default(),
            "I'm looking into it.",
        );
    }

    #[test]
    fn offers_both_suggestions() {
        assert_good_and_bad_suggestions(
            "I've looking into it.",
            IvePerfectProgressive::default(),
            &["I'm looking into it.", "I've been looking into it."],
            &[],
        );
    }

    #[test]
    fn allows_ive_looked() {
        assert_lint_count("I've looked into it.", IvePerfectProgressive::default(), 0);
    }

    #[test]
    fn allows_ive_been_looking() {
        assert_lint_count(
            "I've been looking into it.",
            IvePerfectProgressive::default(),
            0,
        );
    }

    #[test]
    fn allows_ive_seen() {
        assert_lint_count(
            "I've seen the results.",
            IvePerfectProgressive::default(),
            0,
        );
    }

    #[test]
    fn allows_ive_long_been_looking() {
        assert_lint_count(
            "I've long been looking into it.",
            IvePerfectProgressive::default(),
            0,
        );
    }

    #[test]
    fn no_match_with_punctuation_between() {
        assert_lint_count(
            "I've, looking into it.",
            IvePerfectProgressive::default(),
            0,
        );
    }

    #[test]
    fn handles_newline_whitespace() {
        assert_suggestion_result(
            "I've\nlooking into it.",
            IvePerfectProgressive::default(),
            "I'm\nlooking into it.",
        );
    }

    #[test]
    fn capitalization_all_caps_base() {
        assert_suggestion_result(
            "I'VE looking into it.",
            IvePerfectProgressive::default(),
            "I'M looking into it.",
        );
    }

    #[test]
    fn works_for_weve() {
        assert_suggestion_result(
            "We've looking into it.",
            IvePerfectProgressive::default(),
            "We're looking into it.",
        );
    }
}
