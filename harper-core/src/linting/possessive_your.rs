use crate::Token;
use crate::TokenKind;
use crate::expr::Expr;
use crate::expr::SequenceExpr;

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct PossessiveYour {
    expr: Box<dyn Expr>,
}

impl Default for PossessiveYour {
    fn default() -> Self {
        let pattern = SequenceExpr::aco("you")
            .then_whitespace()
            .then_kind_is_but_is_not_except(
                TokenKind::is_nominal,
                TokenKind::is_likely_homograph,
                &["guys", "what's"],
            );

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for PossessiveYour {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.first()?.span;
        let orig_chars = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![
                Suggestion::replace_with_match_case("your".chars().collect(), orig_chars),
                Suggestion::replace_with_match_case("you're a".chars().collect(), orig_chars),
                Suggestion::replace_with_match_case("you're an".chars().collect(), orig_chars),
            ],
            message: "The possessive version of this word is more common in this context."
                .to_owned(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "The possessive form of `you` is more likely before nouns."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{
        assert_lint_count, assert_no_lints, assert_suggestion_result, assert_top3_suggestion_result,
    };

    use super::PossessiveYour;

    #[test]
    #[should_panic] // currently fails because comments is a homographs (verb or noun)
    fn your_comments() {
        assert_suggestion_result(
            "You comments may end up in the documentation.",
            PossessiveYour::default(),
            "Your comments may end up in the documentation.",
        );
    }

    #[test]
    fn allow_intro_page() {
        assert_lint_count(
            "You can try out an editor that uses Harper under-the-hood here.",
            PossessiveYour::default(),
            0,
        );
    }

    #[test]
    fn allow_you_guys() {
        assert_lint_count(
            "I mean I'm pretty sure you guys can't do anything with this stuff.",
            PossessiveYour::default(),
            0,
        );
    }

    #[test]
    fn test_top3_suggestion_your() {
        assert_top3_suggestion_result(
            "You combination of artist and teacher.",
            PossessiveYour::default(),
            "Your combination of artist and teacher.",
        );
    }

    #[test]
    fn test_top3_suggestion_youre_a() {
        assert_top3_suggestion_result(
            "You combination of artist and teacher.",
            PossessiveYour::default(),
            "You're a combination of artist and teacher.",
        );
    }

    #[test]
    #[ignore]
    fn test_top3_suggestion_multiple() {
        assert_top3_suggestion_result(
            "You knowledge. You imagination. You icosahedron",
            PossessiveYour::default(),
            "Your knowledge. Your imagination. You're an icosahedron",
        );
    }

    #[test]
    fn dont_flag_just_showing_you() {
        assert_lint_count(
            "I'm just showing you what's available and how to use it.",
            PossessiveYour::default(),
            0,
        );
    }

    #[test]
    fn allows_issue_1583() {
        assert_no_lints(
            "Note that in a world with modules everywhere, you almost never need an IIFE",
            PossessiveYour::default(),
        );
    }
}
