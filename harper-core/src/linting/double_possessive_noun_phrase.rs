use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct DoublePossessiveNounPhrase {
    expr: SequenceExpr,
}

impl Default for DoublePossessiveNounPhrase {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_possessive_determiner()
                .t_ws()
                .then_adjective()
                .t_ws()
                .t_set([
                    "column", "draft", "edition", "half", "line", "part", "point", "portion",
                    "row", "section", "segment", "verse", "year",
                ])
                .t_ws()
                .t_aco("of")
                .t_ws()
                .then_possessive_determiner(),
        }
    }
}

impl ExprLinter for DoublePossessiveNounPhrase {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let (poss1, poss2) = (toks.first()?, toks.last()?);
        if !poss1.get_ch(src).eq_ch_lenient(poss2.get_ch(src)) {
            return None;
        }
        let span = poss1.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Redundancy,
            suggestions: vec![Suggestion::replace_with_match_case(
                vec!['t', 'h', 'e'],
                span.get_content(src),
            )],
            message: "Avoid the redundant possessive by changing the first to `the`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Detects redundant possessive constructions like `my latter half of my book`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::DoublePossessiveNounPhrase;

    #[test]
    fn fix_my_first_part() {
        assert_suggestion_result(
            "This is my first part of my phrase.",
            DoublePossessiveNounPhrase::default(),
            "This is the first part of my phrase.",
        );
    }

    #[test]
    fn fix_your_second_half() {
        assert_suggestion_result(
            "As you helpfully noted in your second half of your comment, high wealth, deliberately low income",
            DoublePossessiveNounPhrase::default(),
            "As you helpfully noted in the second half of your comment, high wealth, deliberately low income",
        );
    }

    #[test]
    fn fix_his_latter() {
        assert_suggestion_result(
            "In his latter part of his life, he does talk of the thrill",
            DoublePossessiveNounPhrase::default(),
            "In the latter part of his life, he does talk of the thrill",
        );
    }

    #[test]
    fn fix_your_first() {
        assert_suggestion_result(
            "Use this bullet-point template to build out your first draft of your resume.",
            DoublePossessiveNounPhrase::default(),
            "Use this bullet-point template to build out the first draft of your resume.",
        );
    }

    #[test]
    fn fix_your_year() {
        assert_suggestion_result(
            "benefits like 50% off your first year of your subscription",
            DoublePossessiveNounPhrase::default(),
            "benefits like 50% off the first year of your subscription",
        );
    }

    #[test]
    fn dont_flag_inform_of() {
        assert_no_lints(
            "Nash can inform his former clients of his concerns",
            DoublePossessiveNounPhrase::default(),
        );
    }

    #[test]
    fn dont_flag_notify_of() {
        assert_no_lints(
            "it must notify your former spouse of his or her TCC rights",
            DoublePossessiveNounPhrase::default(),
        );
    }

    #[test]
    fn dont_flag_their_of_our() {
        assert_no_lints(
            "Ofgem published their final determination of our business plan.",
            DoublePossessiveNounPhrase::default(),
        );
    }
}
