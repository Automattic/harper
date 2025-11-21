use crate::expr::Expr;
use crate::linting::{ExprLinter, LintKind, Suggestion};
use crate::patterns::Word;
use crate::{Lint, Token};

pub struct PreferSneaked {
    expr: Box<dyn Expr>,
}

impl Default for PreferSneaked {
    fn default() -> Self {
        Self {
            expr: Box::new(Word::new("snuck")),
        }
    }
}

impl ExprLinter for PreferSneaked {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        Some(Lint {
            span: toks[0].span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case(
                ['s', 'n', 'e', 'a', 'k', 'e', 'd'].to_vec(),
                toks[0].span.get_content(src),
            )],
            message: "Use `sneaked` instead of `snuck`.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Prefer 'sneaked' over 'snuck'"
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::sneaked_snuck::PreferSneaked;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn correct_snuck_to_sneaked() {
        assert_suggestion_result(
            "He snuck in around the back.",
            PreferSneaked::default(),
            "He sneaked in around the back.",
        );
    }
}
