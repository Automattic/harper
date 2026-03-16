use crate::{
    Token,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion, expr_linter::Chunk},
    patterns::{InflectionOfBe, Word},
};

pub struct BeWorried {
    expr: All,
}

impl Default for BeWorried {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_subject_pronoun()
                .t_ws()
                .then(InflectionOfBe::default())
                .t_ws()
                .t_aco("worry")
                .and_not(Word::new("it")),
        }
    }
}

impl ExprLinter for BeWorried {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let wtok = matched_tokens.last()?;

        Some(Lint {
            span: wtok.span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "worried",
                wtok.span.get_content(source),
            )],
            message: "Use 'worried' instead of 'worry'.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Detects incorrect use of 'be worry' instead of `be worried`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{
        assert_good_and_bad_suggestions, assert_no_lints, assert_suggestion_result,
    };

    use super::BeWorried;

    #[test]
    fn he_is() {
        assert_suggestion_result(
            "I guess he is worry about \" * user * \" tag.",
            BeWorried::default(),
            "I guess he is worried about \" * user * \" tag.",
        );
    }

    #[test]
    fn i_am() {
        assert_suggestion_result(
            "I didn't see any section dedicated to this so I am worry about:",
            BeWorried::default(),
            "I didn't see any section dedicated to this so I am worried about:",
        );
    }

    // at the same time they are worry about the price for the upgrade each 3 years
    #[test]
    fn they_are() {
        assert_suggestion_result(
            "at the same time they are worry about the price for the upgrade each 3 years",
            BeWorried::default(),
            "at the same time they are worried about the price for the upgrade each 3 years",
        );
    }

    #[test]
    fn we_are() {
        assert_suggestion_result(
            "We are analised this and we are worry because when our platform go to market",
            BeWorried::default(),
            "We are analised this and we are worried because when our platform go to market",
        );
    }

    #[test]
    fn you_are() {
        assert_suggestion_result(
            "You are worry because we are not annotating view interface itself, right?",
            BeWorried::default(),
            "You are worried because we are not annotating view interface itself, right?",
        );
    }

    #[test]
    fn dont_flag_it_is() {
        assert_no_lints(
            "Part of it is worry that my bosses will get angry and fire me.",
            BeWorried::default(),
        );
    }

    #[test]
    #[ignore = "edge case not yet handled"]
    fn cant_fix_edge_case_yet() {
        assert_good_and_bad_suggestions(
            "Myself along with others are using it on an iPad successfully, so it is worry to hear that is broken for you.",
            BeWorried::default(),
            &[
                "Myself along with others are using it on an iPad successfully, so it is worrying to hear that is broken for you.",
                "Myself along with others are using it on an iPad successfully, so it is a worry to hear that is broken for you.",
            ],
            &[],
        );
    }
}
