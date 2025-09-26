use crate::expr::Expr;
use crate::expr::OwnedExprExt;
use crate::expr::SequenceExpr;
use crate::{
    Token,
    patterns::{NominalPhrase, Word},
};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct ForNoun {
    expr: Box<dyn Expr>,
}

impl Default for ForNoun {
    fn default() -> Self {
        let pattern = SequenceExpr::aco("fro")
            .then_whitespace()
            .then(NominalPhrase.or(Word::new("sure")));

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for ForNoun {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.first()?.span;
        let problem_chars = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "for",
                problem_chars,
            )],
            message: "`For` is more common in this context.".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects the archaic or mistaken `fro` to `for` when followed by a noun."
    }
}

#[cfg(test)]
mod tests {
    use super::ForNoun;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_fro_basic_correction() {
        assert_suggestion_result(
            "I got a text fro Sarah.",
            ForNoun::default(),
            "I got a text for Sarah.",
        );
    }

    #[test]
    fn allows_for_clean() {
        assert_lint_count("I got a text for Sarah.", ForNoun::default(), 0);
    }

    #[test]
    fn corrects_fro_sure() {
        assert_suggestion_result(
            "He was away fro sure!",
            ForNoun::default(),
            "He was away for sure!",
        );
    }
}
