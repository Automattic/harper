use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::{Token, TokenKind};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct IveGerund {
    expr: Box<dyn Expr>,
}

impl Default for IveGerund {
    fn default() -> Self {
        let expr = SequenceExpr::aco("I've")
            .t_ws()
            .then_kind_both(TokenKind::is_verb, TokenKind::is_verb_progressive_form);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for IveGerund {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let ive_tok = toks.iter().find(|t| t.kind.is_word())?;
        let span = ive_tok.span;

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![
                Suggestion::replace_with_match_case("I'm".chars().collect(), span.get_content(src)),
                Suggestion::InsertAfter(" been".chars().collect()),
            ],
            message: "Use present progressive (`I'm …`) or present perfect progressive (`I've been …`) instead of `I've …ing`.".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Detects the ungrammatical pattern `I've` followed directly by a gerund (e.g., `I've looking`) and suggests either the present progressive (`I'm …`) or the present perfect progressive (`I've been …`)."
    }
}

#[cfg(test)]
mod tests {
    use super::IveGerund;
    use crate::linting::tests::{
        assert_good_and_bad_suggestions, assert_lint_count, assert_suggestion_result,
    };

    #[test]
    fn suggests_im_looking() {
        assert_suggestion_result(
            "I've looking into it.",
            IveGerund::default(),
            "I'm looking into it.",
        );
    }

    #[test]
    fn corrects_basic_im() {
        assert_suggestion_result(
            "I've looking into it.",
            IveGerund::default(),
            "I'm looking into it.",
        );
    }

    #[test]
    fn offers_both_suggestions() {
        assert_good_and_bad_suggestions(
            "I've looking into it.",
            IveGerund::default(),
            &["I'm looking into it.", "I've been looking into it."],
            &[],
        );
    }

    #[test]
    fn allows_ive_looked() {
        assert_lint_count("I've looked into it.", IveGerund::default(), 0);
    }

    #[test]
    fn allows_ive_been_looking() {
        assert_lint_count("I've been looking into it.", IveGerund::default(), 0);
    }

    #[test]
    fn allows_ive_seen() {
        assert_lint_count("I've seen the results.", IveGerund::default(), 0);
    }

    #[test]
    fn allows_ive_long_been_looking() {
        assert_lint_count("I've long been looking into it.", IveGerund::default(), 0);
    }

    #[test]
    fn no_match_with_punctuation_between() {
        assert_lint_count("I've, looking into it.", IveGerund::default(), 0);
    }

    #[test]
    fn handles_newline_whitespace() {
        assert_suggestion_result(
            "I've\nlooking into it.",
            IveGerund::default(),
            "I'm\nlooking into it.",
        );
    }

    #[test]
    fn capitalization_all_caps_base() {
        assert_suggestion_result(
            "I'VE looking into it.",
            IveGerund::default(),
            "I'M looking into it.",
        );
    }
}
