use crate::{
    Span, Token,
    expr::{Expr, SequenceExpr},
    linting::expr_linter::Chunk,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::DerivedFrom,
};

pub struct CureFor {
    expr: Box<dyn Expr>,
}

impl Default for CureFor {
    fn default() -> Self {
        let expr = SequenceExpr::with(DerivedFrom::new_from_str("cure"))
            .t_ws()
            .t_aco("against");

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for CureFor {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let against = matched_tokens.last()?;

        let template: Vec<char> = against.span.get_content(source).to_vec();
        let suggestion = Suggestion::replace_with_match_case_str("for", &template);

        Some(Lint {
            span: Span::new(against.span.start, against.span.end),
            lint_kind: LintKind::Usage,
            suggestions: vec![suggestion],
            message: "Prefer `cure for` when describing a treatment target.".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Flags `cure against` and prefers the standard `cure for` pairing."
    }
}

#[cfg(test)]
mod tests {
    use super::CureFor;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_simple_cure_against() {
        assert_suggestion_result(
            "Researchers sought a cure against the stubborn illness.",
            CureFor::default(),
            "Researchers sought a cure for the stubborn illness.",
        );
    }

    #[test]
    fn corrects_plural_cures_against() {
        assert_suggestion_result(
            "Doctors insist this serum cures against the new variant.",
            CureFor::default(),
            "Doctors insist this serum cures for the new variant.",
        );
    }

    #[test]
    fn corrects_past_participle_cured_against() {
        assert_suggestion_result(
            "The remedy was cured against the infection last spring.",
            CureFor::default(),
            "The remedy was cured for the infection last spring.",
        );
    }

    #[test]
    fn corrects_uppercase_against() {
        assert_suggestion_result(
            "We still trust the cure AGAINST the dreaded plague.",
            CureFor::default(),
            "We still trust the cure FOR the dreaded plague.",
        );
    }

    #[test]
    fn corrects_at_sentence_start() {
        assert_suggestion_result(
            "Cure against that condition became the rallying cry.",
            CureFor::default(),
            "Cure for that condition became the rallying cry.",
        );
    }

    #[test]
    fn does_not_flag_cure_for() {
        assert_lint_count(
            "They finally found a cure for the fever.",
            CureFor::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_cure_from() {
        assert_lint_count(
            "A cure from this rare herb is on the horizon.",
            CureFor::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_with_comma() {
        assert_lint_count(
            "A cure, against all odds, appeared in the files.",
            CureFor::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_unrelated_against() {
        assert_lint_count(
            "Travelers stand against the roaring wind on the cliffs.",
            CureFor::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_secure_against() {
        assert_lint_count(
            "The fortress stayed secure against the invaders.",
            CureFor::default(),
            0,
        );
    }
}
