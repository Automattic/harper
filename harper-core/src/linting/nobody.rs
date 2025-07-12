use crate::TokenStringExt;
use crate::expr::Expr;
use crate::expr::MatchInfo;
use crate::expr::SequenceExpr;

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct Nobody {
    expr: Box<dyn Expr>,
}

impl Default for Nobody {
    fn default() -> Self {
        let pattern = SequenceExpr::aco("no")
            .then_whitespace()
            .t_aco("body")
            .then_whitespace()
            .then_verb();
        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for Nobody {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, match_info: MatchInfo<'_>, source: &[char]) -> Option<Lint> {
        let matched_tokens = match_info.matched_tokens;
        let span = matched_tokens[0..3].span()?;
        let orig_chars = span.get_content(source);
        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                "nobody".chars().collect(),
                orig_chars,
            )],
            message: format!("Did you mean the closed compound `{}`?", "nobody"),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Looks for incorrect spacing inside the closed compound `nobody`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::Nobody;

    #[test]
    fn both_valid_and_invalid() {
        assert_suggestion_result(
            "No body told me. I have a head but no body.",
            Nobody::default(),
            "Nobody told me. I have a head but no body.",
        );
    }
}
