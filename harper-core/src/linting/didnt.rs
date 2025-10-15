use crate::Token;
use crate::expr::{Expr, SequenceExpr};
use crate::linting::{ExprLinter, Lint, LintKind, Suggestion};

pub struct Didnt {
    expr: Box<dyn Expr>,
}

impl Default for Didnt {
    fn default() -> Self {
        let pattern = SequenceExpr::default()
            .then_personal_pronoun()
            .t_ws()
            .t_aco("dint");

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for Didnt {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let suspect = toks.last()?;

        Some(Lint {
            span: suspect.span,
            lint_kind: LintKind::Typo,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "didn't",
                suspect.span.get_content(src),
            )],
            message: "Consider using `didn't` here.".to_string(),
            priority: 63,
        })
    }

    fn description(&self) -> &str {
        "Corrects `dint` to `didn't` after subject pronouns."
    }
}

#[cfg(test)]
mod tests {
    use super::Didnt;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn corrects_i_dint() {
        assert_suggestion_result(
            "I dint lock the gate.",
            Didnt::default(),
            "I didn't lock the gate.",
        );
    }

    #[test]
    fn corrects_you_dint() {
        assert_suggestion_result(
            "You dint look this way.",
            Didnt::default(),
            "You didn't look this way.",
        );
    }

    #[test]
    fn corrects_he_dint() {
        assert_suggestion_result(
            "He dint see the sign.",
            Didnt::default(),
            "He didn't see the sign.",
        );
    }

    #[test]
    fn corrects_she_dint() {
        assert_suggestion_result(
            "She dint call me back.",
            Didnt::default(),
            "She didn't call me back.",
        );
    }

    #[test]
    fn corrects_we_dint() {
        assert_suggestion_result(
            "We dint sleep much.",
            Didnt::default(),
            "We didn't sleep much.",
        );
    }

    #[test]
    fn corrects_they_dint() {
        assert_suggestion_result(
            "They dint enjoy the show.",
            Didnt::default(),
            "They didn't enjoy the show.",
        );
    }

    #[test]
    fn corrects_it_dint() {
        assert_suggestion_result(
            "It dint rain today.",
            Didnt::default(),
            "It didn't rain today.",
        );
    }

    #[test]
    fn does_not_flag_dint_noun() {
        assert_no_lints("The blow left a small dint in the metal.", Didnt::default());
    }

    #[test]
    fn does_not_flag_quoted_dint() {
        assert_no_lints("He muttered 'dint' under his breath.", Didnt::default());
    }

    #[test]
    fn does_not_flag_past_tense_with_not() {
        assert_lint_count("I did not lock the gate.", Didnt::default(), 0);
    }
}
