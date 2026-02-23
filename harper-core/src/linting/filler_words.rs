use crate::linting::expr_linter::Chunk;
use crate::{
    Lrc, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

pub struct FillerWords {
    expr: Box<dyn Expr>,
}

impl Default for FillerWords {
    // A filler is unlikely to be completely on its own, so check for and remove with whitespace either before or after.
    fn default() -> Self {
        let filler_words = Lrc::new(WordSet::new(&["uh", "um"]));

        let pattern = SequenceExpr::any_of(vec![
            Box::new(SequenceExpr::with(filler_words.clone()).then_whitespace()),
            Box::new(SequenceExpr::whitespace().then(filler_words)),
        ]);

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for FillerWords {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
        // A filler is unlikely to be completely on its own, so check for and remove with whitespace either before or after.
        Some(Lint {
            span: toks.span()?,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::Remove],
            message: "Remove this unnecessary filler word.".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Removes filler words."
    }
}

#[cfg(test)]
mod tests {
    use super::FillerWords;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn remove_uh() {
        assert_suggestion_result(
            "Let's remove all the uh filler words.",
            FillerWords::default(),
            "Let's remove all the filler words.",
        );
    }

    #[test]
    fn remove_um_st_start() {
        assert_suggestion_result(
            "Um but I'll just add some context for this.",
            FillerWords::default(),
            "but I'll just add some context for this.",
        );
    }
}
