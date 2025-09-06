use crate::TokenStringExt;
use crate::expr::Expr;
use crate::expr::MatchInfo;
use crate::expr::SequenceExpr;

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct Whereas {
    expr: Box<dyn Expr>,
}

impl Default for Whereas {
    fn default() -> Self {
        let pattern = SequenceExpr::default()
            .t_aco("where")
            .then_whitespace()
            .t_aco("as");

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for Whereas {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, match_info: MatchInfo<'_>, source: &[char]) -> Option<Lint> {
        let matched_tokens = match_info.matched_tokens;
        let span = matched_tokens.span()?;
        let orig_chars = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                vec!['w', 'h', 'e', 'r', 'e', 'a', 's'],
                orig_chars,
            )],
            message: "`Whereas` is commonly mistaken for `where as`.".to_owned(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "The Whereas rule is designed to identify instances where the phrase `where as` is used in text and suggests replacing it with the single word `whereas`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::Whereas;

    #[test]
    fn where_as() {
        assert_suggestion_result(
            "Dogs love playing fetch, where as cats are more independent creatures.",
            Whereas::default(),
            "Dogs love playing fetch, whereas cats are more independent creatures.",
        );
    }
}
