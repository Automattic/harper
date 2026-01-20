use crate::expr::Expr;
use crate::linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk};
use crate::patterns::Word;
use crate::{Lint, Token};

pub struct PreferSnuck {
    expr: Box<dyn Expr>,
}

impl Default for PreferSnuck {
    fn default() -> Self {
        Self {
            expr: Box::new(Word::new("sneaked")),
        }
    }
}

impl ExprLinter for PreferSnuck {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        Some(Lint {
            span: toks[0].span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case(
                ['s', 'n', 'u', 'c', 'k'].to_vec(),
                toks[0].span.get_content(src),
            )],
            message: "Use `snuck` instead of `sneaked`.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Prefer `snuck` over `sneaked`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::sneaked_snuck::PreferSnuck;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn correct_sneaked_to_snuck() {
        assert_suggestion_result(
            "He sneaked in around the back.",
            PreferSnuck::default(),
            "He snuck in around the back.",
        );
    }
}
