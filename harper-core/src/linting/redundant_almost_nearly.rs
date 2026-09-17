use crate::linting::expr_linter::Chunk;
use crate::{
    Lrc, Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, FixedPhrase, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

pub struct RedundantAlmostNearly {
    expr: FirstMatchOf,
}

impl Default for RedundantAlmostNearly {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new(vec![
                Box::new(SequenceExpr::word_seq(&["almost", "nearly"])),
                Box::new(SequenceExpr::word_seq(&["nearly", "almost"])),
            ]),
        }
    }
}

impl ExprLinter for RedundantAlmostNearly {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.span()?;
        Some(Lint {
            span,
            lint_kind: LintKind::Redundancy,
            suggestions: vec![
                Suggestion::replace_with_match_case_str("almost", span.get_content(src)),
                Suggestion::replace_with_match_case_str("nearly", span.get_content(src)),
            ],
            message: "Use just one of `almost` or `nearly`.".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Detects `almost` and `nearly` redundantly used together."
    }
}

#[cfg(test)]
mod tests {
    use super::RedundantAlmostNearly;
    use crate::linting::tests::assert_good_and_bad_suggestions;

    #[test]
    fn almost_nearly() {
        assert_good_and_bad_suggestions(
            "Sub-word encoding models are almost nearly identical to the Character encoding models.",
            RedundantAlmostNearly::default(),
            &[
                "Sub-word encoding models are almost identical to the Character encoding models.",
                "Sub-word encoding models are nearly identical to the Character encoding models.",
            ],
            &[],
        );
    }

    #[test]
    fn nearly_almost() {
        assert_good_and_bad_suggestions(
            "Running Openshift 3.11 inside WSL2. This is not running yet, but nearly almost.",
            RedundantAlmostNearly::default(),
            &[
                "Running Openshift 3.11 inside WSL2. This is not running yet, but almost.",
                "Running Openshift 3.11 inside WSL2. This is not running yet, but nearly.",
            ],
            &[],
        );
    }
}
