use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::{Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct DeterminerWithoutNoun {
    expr: Box<dyn Expr>,
}

impl Default for DeterminerWithoutNoun {
    fn default() -> Self {
        let expr = SequenceExpr::default()
            .then(|tok: &Token, _: &[char]| tok.kind.is_determiner())
            .t_ws()
            .then_conjunction();

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for DeterminerWithoutNoun {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: Vec::<Suggestion>::new(),
            message: "A determiner should not be immediately followed by a conjunction."
                .to_string(),
            priority: 32,
        })
    }

    fn description(&self) -> &'static str {
        "Flags sequences where a determiner (`a`, `an`, `the`, etc.) is directly followed by a coordinating or subordinating conjunction (`and`, `or`, `but`, `because`, etc.), indicating a missing noun."
    }
}

#[cfg(test)]
mod tests {
    use super::DeterminerWithoutNoun;
    use crate::linting::tests::assert_lint_count;

    #[test]
    fn flags_determiner_followed_by_conjunction() {
        assert_lint_count(
            "The and other options were ignored.",
            DeterminerWithoutNoun::default(),
            1,
        );
    }

    #[test]
    fn flags_indefinite_article_followed_by_conjunction() {
        assert_lint_count("A because I said so.", DeterminerWithoutNoun::default(), 1);
        assert_lint_count("An because I said so.", DeterminerWithoutNoun::default(), 1);
    }

    #[test]
    fn allows_correct_use_with_noun() {
        assert_lint_count("The dog barked.", DeterminerWithoutNoun::default(), 0);
    }

    #[test]
    fn allows_determiner_noun_then_conjunction() {
        assert_lint_count(
            "The dog and the cat played.",
            DeterminerWithoutNoun::default(),
            0,
        );
    }
}
