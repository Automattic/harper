use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::{
    Token,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

pub struct ItsContraction {
    expr: Box<dyn Expr>,
}

impl Default for ItsContraction {
    fn default() -> Self {
        let its = WordSet::new(&["its"]);
        let verbs = WordSet::new(&["had", "been", "got"]);
        let pattern = SequenceExpr::default()
            .then(its)
            .then_whitespace()
            .then(verbs);
        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for ItsContraction {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], source: &[char]) -> Option<Lint> {
        let offender = toks.first()?;
        let offender_chars = offender.span.get_content(source);
        Some(Lint {
            span: offender.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![
                Suggestion::replace_with_match_case_str("it's", offender_chars),
                Suggestion::replace_with_match_case_str("it has", offender_chars),
            ],
            message: "Use `it's` (short for `it has`) here, not the possessive `its`.".to_owned(),
            priority: 54,
        })
    }

    fn description(&self) -> &str {
        "Detects the possessive `its` before `had`, `been`, or `got` and offers `it's` or `it has`."
    }
}

#[cfg(test)]
mod tests {
    use super::ItsContraction;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn fix_had() {
        assert_suggestion_result(
            "Its had an enormous effect.",
            ItsContraction::default(),
            "It's had an enormous effect.",
        );
    }

    #[test]
    fn fix_been() {
        assert_suggestion_result(
            "Its been months since we spoke.",
            ItsContraction::default(),
            "It's been months since we spoke.",
        );
    }

    #[test]
    fn fix_got() {
        assert_suggestion_result(
            "I think its got nothing to do with us.",
            ItsContraction::default(),
            "I think it's got nothing to do with us.",
        );
    }

    #[test]
    fn ignore_correct_contraction() {
        assert_lint_count(
            "It's been a long year for everyone.",
            ItsContraction::default(),
            0,
        );
    }

    #[test]
    fn ignore_possessive() {
        assert_lint_count(
            "The company revised its policies last week.",
            ItsContraction::default(),
            0,
        );
    }
}
