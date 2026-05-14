use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::{Token, TokenKind, TokenStringExt};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct Hereby {
    expr: SequenceExpr,
}

impl Default for Hereby {
    fn default() -> Self {
        // Require a verb that is not also a noun. Otherwise sentences like
        // "I got here by skill" — where "skill" is a noun object of "by" —
        // match because "skill" is tagged as both verb and noun.
        let pattern = SequenceExpr::aco("here")
            .then_whitespace()
            .t_aco("by")
            .then_whitespace()
            .then_kind_is_but_is_not(TokenKind::is_verb, TokenKind::is_noun);

        Self { expr: pattern }
    }
}

impl ExprLinter for Hereby {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens[0..3].span()?;
        let orig_chars = span.get_content(source);
        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                "hereby".chars().collect(),
                orig_chars,
            )],
            message: "Did you mean the closed compound `hereby`?".to_owned(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "`Here by` in some contexts should be `hereby`"
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::Hereby;

    #[test]
    fn declare() {
        assert_suggestion_result(
            "I here by declare this state to be free.",
            Hereby::default(),
            "I hereby declare this state to be free.",
        );
    }

    #[test]
    fn allows_here_by_noun() {
        use crate::linting::tests::assert_no_lints;
        assert_no_lints("I got here by skill.", Hereby::default());
    }
}
