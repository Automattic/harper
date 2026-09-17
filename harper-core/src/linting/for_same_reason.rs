use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::WordSet,
};

enum Remedy {
    InsertDefiniteArticle,
    Pluralize,
}
use Remedy::*;

const MODIFIERS: &[(&str, Remedy)] = &[
    ("financial", Pluralize),
    ("following", InsertDefiniteArticle),
    // "for incorrect reason" is shorthand, not a mistake
    ("obvious", Pluralize),
    ("other", Pluralize),
    ("same", InsertDefiniteArticle),
    ("security", Pluralize),
    ("similar", Pluralize),
    ("special", Pluralize),
    ("such", Pluralize),
    // "for unknown reason" is shorthand, not a mistake
    ("various", Pluralize),
];

pub struct ForSameReason {
    expr: SequenceExpr,
}

impl Default for ForSameReason {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("for")
                .t_ws()
                .then(MODIFIERS.iter().map(|(w, _)| *w).collect::<WordSet>())
                .t_ws()
                .t_aco("reason"),
        }
    }
}

impl ExprLinter for ForSameReason {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let (ftok, mtok, rtok) = (&toks[0], &toks[2], &toks[4]);
        let (fspan, rspan) = (ftok.span, rtok.span);
        let mod_str = mtok.get_ch(src);

        let remedy = MODIFIERS
            .iter()
            .find(|(m, _)| mod_str.eq_str(m))
            .map(|(_, r)| r)?;

        let (span, sugg, msg) = match remedy {
            InsertDefiniteArticle => (
                fspan,
                Suggestion::replace_with_match_case_str("for the", ftok.get_ch(src)),
                "insert `the` before `reason`",
            ),
            Pluralize => (
                rspan,
                Suggestion::replace_with_match_case_str("reasons", rtok.get_ch(src)),
                "use the plural `reasons`",
            ),
        };

        Some(Lint {
            span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![sugg],
            message: format!("In this context, {msg}.").to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "A linter for detecting incorrect use of `reason` vs `reasons` in certain contexts."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ForSameReason;

    #[test]
    fn for_financial_reasons() {
        assert_suggestion_result(
            "... tries to hide the real cause of death or for financial reason, ...",
            ForSameReason::default(),
            "... tries to hide the real cause of death or for financial reasons, ...",
        );
    }

    #[test]
    fn dont_flag_for_human_reason() {
        assert_no_lints(
            "the platonic ideal of reasoning is not even an approximation for human reason",
            ForSameReason::default(),
        );
    }

    #[test]
    fn for_obvious_reasons() {
        assert_suggestion_result(
            "this will add 1-bit per cycle, but no body uses this for obvious reason, it's just a toy example ...",
            ForSameReason::default(),
            "this will add 1-bit per cycle, but no body uses this for obvious reasons, it's just a toy example ...",
        );
    }

    #[test]
    fn for_obvious_reasons_2() {
        assert_suggestion_result(
            "it is certainly not the same system, for obvious reason",
            ForSameReason::default(),
            "it is certainly not the same system, for obvious reasons",
        );
    }

    #[test]
    fn for_other_reasons() {
        assert_suggestion_result(
            "sometimes as I touched them for other reason and other times as independent changes",
            ForSameReason::default(),
            "sometimes as I touched them for other reasons and other times as independent changes",
        );
    }

    #[test]
    fn for_other_reasons_2() {
        assert_suggestion_result(
            "but it might be greyed out for other reason though",
            ForSameReason::default(),
            "but it might be greyed out for other reasons though",
        );
    }

    #[test]
    fn for_the_same_reason() {
        assert_suggestion_result(
            "we tried and failed for same reason",
            ForSameReason::default(),
            "we tried and failed for the same reason",
        );
    }

    #[test]
    fn for_the_same_reason_2() {
        assert_suggestion_result(
            "For same reason AI controled surgery isn't going to replace surgical school experience",
            ForSameReason::default(),
            "For the same reason AI controled surgery isn't going to replace surgical school experience",
        );
    }

    #[test]
    fn for_security_reasons() {
        assert_suggestion_result(
            "HTTPS is required for security reason",
            ForSameReason::default(),
            "HTTPS is required for security reasons",
        );
    }

    #[test]
    fn for_similar_reasons() {
        assert_suggestion_result(
            "For similar reason you can't use Earth's magnetic field",
            ForSameReason::default(),
            "For similar reasons you can't use Earth's magnetic field",
        );
    }

    #[test]
    fn for_similar_reasons_2() {
        assert_suggestion_result(
            "This is, in some sense, for similar reason to Gunnar Carlson et al finding a Klein bottle",
            ForSameReason::default(),
            "This is, in some sense, for similar reasons to Gunnar Carlson et al finding a Klein bottle",
        );
    }

    #[test]
    fn for_various_reasons() {
        assert_suggestion_result(
            "We decided for various reason to move back to Finland",
            ForSameReason::default(),
            "We decided for various reasons to move back to Finland",
        );
    }
}
