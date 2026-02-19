use crate::expr::Expr;
use crate::expr::OwnedExprExt;
use crate::expr::SequenceExpr;
use crate::linting::expr_linter::Chunk;
use crate::{
    Token,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::{InflectionOfBe, Word},
};

pub struct SaveToSafe {
    expr: Box<dyn Expr>,
}

impl Default for SaveToSafe {
    fn default() -> Self {
        let pattern = SequenceExpr::with(InflectionOfBe::new().or(Word::new("it")))
            .then_whitespace()
            .t_aco("save")
            .then_whitespace()
            .t_aco("to")
            .then_whitespace()
            .then_verb();
        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for SaveToSafe {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let save_tok = &toks.get(2)?;
        let verb_tok = &toks.get(4)?;
        let verb = verb_tok.span.get_content_string(src).to_lowercase();
        Some(Lint {
            span: save_tok.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::ReplaceWith("safe".chars().collect())],
            message: format!("Did you mean `safe to {verb}`?"),
            priority: 57,
        })
    }

    fn description(&self) -> &str {
        "Corrects `save to <verb>` to `safe to <verb>` after a form of `be`."
    }
}

#[cfg(test)]
mod tests {
    use super::SaveToSafe;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn fix_ignore() {
        assert_suggestion_result(
            "It is save to ignore trivial code.",
            SaveToSafe::default(),
            "It is safe to ignore trivial code.",
        );
    }

    #[test]
    fn fix_travel() {
        assert_suggestion_result(
            "Is it save to travel abroad now?",
            SaveToSafe::default(),
            "Is it safe to travel abroad now?",
        );
    }

    #[test]
    fn ignore_correct() {
        assert_lint_count("It is safe to assume nothing.", SaveToSafe::default(), 0);
    }
}
